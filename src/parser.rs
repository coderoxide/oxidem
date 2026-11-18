use crate::ast::*;
use crate::span::Span;
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    fn expect(&mut self, expected: TokenKind) -> Result<Token, String> {
        let current = self.peek().clone();
        if std::mem::discriminant(&current.kind) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(current)
        } else {
            Err(format!(
                "Hata [{}:{}]: Beklenen '{:?}', bulunan '{:?}'",
                current.span.line, current.span.col, expected, current.kind
            ))
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            if self.check(&TokenKind::Import) {
                let import_span = self.advance().span;
                let path = match self.peek().kind.clone() {
                    TokenKind::StringLit(s) => {
                        self.advance();
                        s
                    }
                    _ => {
                        return Err(format!(
                            "Hata [{}:{}]: 'import' ifadesinden sonra dosya yolu metin dizgisi (\"dosya.ox\") bekleniyordu",
                            import_span.line, import_span.col
                        ));
                    }
                };
                self.expect(TokenKind::Semicolon)?;
                items.push(Item::Import(path, import_span));
            } else if self.check(&TokenKind::Const) {
                items.push(Item::Const(self.parse_const()?));
            } else if self.check(&TokenKind::Fn) {
                items.push(Item::Function(self.parse_function()?));
            } else if self.check(&TokenKind::Struct) {
                items.push(Item::Struct(self.parse_struct()?));
            } else if self.check(&TokenKind::Impl) {
                items.push(Item::Impl(self.parse_impl()?));
            } else if self.check(&TokenKind::Trait) {
                items.push(Item::Trait(self.parse_trait()?));
            } else if self.check(&TokenKind::Enum) {
                items.push(Item::Enum(self.parse_enum()?));
            } else if self.check(&TokenKind::Extern) {
                items.push(Item::Extern(self.parse_extern_block()?));
            } else {
                let token = self.peek();
                return Err(format!(
                    "Hata [{}:{}]: Dosya seviyesinde yalnızca 'import', 'const', 'extern', 'fn', 'struct', 'impl', 'trait' veya 'enum' bildirimlerine izin verilir. Bulunan: {:?}",
                    token.span.line, token.span.col, token.kind
                ));
            }
        }

        Ok(Program { items })
    }

    fn parse_extern_block(&mut self) -> Result<ExternBlock, String> {
        let extern_span = self.advance().span;
        let abi = match self.peek().kind.clone() {
            TokenKind::StringLit(s) => {
                self.advance();
                s
            }
            _ => "C".to_string(),
        };
        self.expect(TokenKind::LBrace)?;
        let mut functions = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let fn_span = self.expect(TokenKind::Fn)?.span;
            let name = self.expect_ident()?;
            self.expect(TokenKind::LParen)?;
            let mut params = Vec::new();
            if !self.check(&TokenKind::RParen) {
                loop {
                    let is_mut = if self.check(&TokenKind::Mut) {
                        self.advance();
                        true
                    } else {
                        false
                    };
                    let param_name = self.expect_ident()?;
                    self.expect(TokenKind::Colon)?;
                    let param_ty = self.parse_type()?;
                    params.push(Param {
                        name: param_name,
                        ty: param_ty,
                        is_mut,
                    });
                    if self.check(&TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen)?;
            let return_type = if self.check(&TokenKind::Arrow) {
                self.advance();
                self.parse_type()?
            } else {
                Type::Void
            };
            self.expect(TokenKind::Semicolon)?;
            functions.push(ExternFn {
                name,
                params,
                return_type,
                span: fn_span,
            });
        }
        self.expect(TokenKind::RBrace)?;
        Ok(ExternBlock {
            abi,
            functions,
            span: extern_span,
        })
    }

    fn parse_enum(&mut self) -> Result<EnumDef, String> {
        let enum_tok = self.expect(TokenKind::Enum)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut variants = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let variant = self.expect_ident()?;
            variants.push(variant);

            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(EnumDef {
            name,
            variants,
            span: enum_tok.span,
        })
    }

    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>, String> {
        let mut generic_params = Vec::new();
        if self.check(&TokenKind::Lt) {
            self.advance();
            while !self.check(&TokenKind::Gt) && !self.is_at_end() {
                let name = self.expect_ident()?;
                let mut bounds = Vec::new();
                if self.check(&TokenKind::Colon) {
                    self.advance();
                    bounds.push(self.expect_ident()?);
                    while self.check(&TokenKind::Plus) {
                        self.advance();
                        bounds.push(self.expect_ident()?);
                    }
                }
                generic_params.push(GenericParam { name, bounds });
                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::Gt)?;
        }
        Ok(generic_params)
    }

    fn parse_struct(&mut self) -> Result<StructDef, String> {
        let struct_tok = self.expect(TokenKind::Struct)?;
        let name = self.expect_ident()?;
        let generic_params = self.parse_generic_params()?;
        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let field_name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let field_ty = self.parse_type()?;
            fields.push((field_name, field_ty));

            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(StructDef {
            name,
            generic_params,
            fields,
            span: struct_tok.span,
        })
    }

    fn parse_impl(&mut self) -> Result<ImplBlock, String> {
        let impl_tok = self.expect(TokenKind::Impl)?;
        let generic_params = self.parse_generic_params()?;
        let first_name = self.expect_ident()?;
        let (trait_name, target) = if self.check(&TokenKind::For) {
            self.advance();
            let target = self.expect_ident()?;
            (Some(first_name), target)
        } else {
            (None, first_name)
        };

        if self.check(&TokenKind::Lt) {
            self.advance();
            while !self.check(&TokenKind::Gt) && !self.is_at_end() {
                self.parse_type()?;
                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::Gt)?;
        }
        self.expect(TokenKind::LBrace)?;

        let mut methods = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            methods.push(self.parse_function()?);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(ImplBlock {
            trait_name,
            target,
            generic_params,
            methods,
            span: impl_tok.span,
        })
    }

    fn parse_trait(&mut self) -> Result<TraitDef, String> {
        let trait_tok = self.expect(TokenKind::Trait)?;
        let name = self.expect_ident()?;
        let generic_params = self.parse_generic_params()?;
        self.expect(TokenKind::LBrace)?;

        let mut methods = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let fn_span = self.expect(TokenKind::Fn)?.span;
            let m_name = self.expect_ident()?;
            self.expect(TokenKind::LParen)?;
            let mut params = Vec::new();
            if !self.check(&TokenKind::RParen) {
                loop {
                    let is_mut = if self.check(&TokenKind::Mut) {
                        self.advance();
                        true
                    } else {
                        false
                    };
                    let param_name = self.expect_ident()?;
                    let param_ty = if param_name == "self" && !self.check(&TokenKind::Colon) {
                        Type::Named("Self".to_string())
                    } else {
                        self.expect(TokenKind::Colon)?;
                        self.parse_type()?
                    };
                    params.push(Param {
                        name: param_name,
                        ty: param_ty,
                        is_mut,
                    });
                    if self.check(&TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen)?;
            let return_type = if self.check(&TokenKind::Arrow) {
                self.advance();
                self.parse_type()?
            } else {
                Type::Void
            };

            let default_body = if self.check(&TokenKind::LBrace) {
                Some(self.parse_block()?)
            } else {
                self.expect(TokenKind::Semicolon)?;
                None
            };

            methods.push(TraitMethod {
                name: m_name,
                params,
                return_type,
                default_body,
                span: fn_span,
            });
        }

        self.expect(TokenKind::RBrace)?;
        Ok(TraitDef {
            name,
            generic_params,
            methods,
            span: trait_tok.span,
        })
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        let fn_token = self.expect(TokenKind::Fn)?;
        let name_token = self.expect_ident()?;
        let name = name_token;
        let generic_params = self.parse_generic_params()?;

        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                let is_mut = if self.check(&TokenKind::Mut) {
                    self.advance();
                    true
                } else {
                    false
                };
                let param_name = self.expect_ident()?;
                let param_ty = if param_name == "self" && !self.check(&TokenKind::Colon) {
                    Type::Named("Self".to_string())
                } else {
                    self.expect(TokenKind::Colon)?;
                    self.parse_type()?
                };
                let is_mut = is_mut || param_name == "self";
                params.push(Param {
                    name: param_name,
                    ty: param_ty,
                    is_mut,
                });

                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen)?;

        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = self.parse_block()?;

        Ok(Function {
            name,
            generic_params,
            params,
            return_type,
            body,
            span: fn_token.span,
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        if self.check(&TokenKind::LParen) {
            self.advance();
            let mut types = Vec::new();
            let mut has_comma = false;
            if !self.check(&TokenKind::RParen) {
                loop {
                    types.push(self.parse_type()?);
                    if self.check(&TokenKind::Comma) {
                        self.advance();
                        has_comma = true;
                        if self.check(&TokenKind::RParen) {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen)?;
            if types.is_empty() {
                return Ok(Type::Void);
            } else if types.len() == 1 && !has_comma {
                return Ok(types.pop().unwrap());
            } else {
                return Ok(Type::Tuple(types));
            }
        }

        if self.check(&TokenKind::Fn) {
            self.advance();
            self.expect(TokenKind::LParen)?;
            let mut params = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                params.push(self.parse_type()?);
                if self.check(&TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
            let ret = if self.check(&TokenKind::Arrow) {
                self.advance();
                self.parse_type()?
            } else {
                Type::Void
            };
            return Ok(Type::Fn {
                params,
                ret: Box::new(ret),
            });
        }

        if self.check(&TokenKind::Ampersand) {
            self.advance();
            let is_mut = if self.check(&TokenKind::Mut) {
                self.advance();
                true
            } else {
                false
            };
            if self.check(&TokenKind::LBracket) {
                self.advance();
                let inner = self.parse_type()?;
                self.expect(TokenKind::RBracket)?;
                return Ok(Type::Slice(Box::new(inner)));
            }
            let inner = self.parse_type()?;
            if inner == Type::Str {
                return Ok(Type::StrSlice);
            }
            return Ok(Type::Ref {
                inner: Box::new(inner),
                is_mut,
            });
        }

        if self.check(&TokenKind::Star) {
            let star_span = self.advance().span;
            let is_mut = if self.check(&TokenKind::Mut) {
                self.advance();
                true
            } else if self.check(&TokenKind::Const) {
                self.advance();
                false
            } else {
                return Err(format!(
                    "Hata [{}:{}]: Ham gösterici (raw pointer) '*const' veya '*mut' olarak belirtilmelidir",
                    star_span.line, star_span.col
                ));
            };
            let inner = self.parse_type()?;
            return Ok(Type::RawPtr {
                inner: Box::new(inner),
                is_mut,
            });
        }

        if self.check(&TokenKind::LBracket) {
            self.advance();
            let inner = self.parse_type()?;
            self.expect(TokenKind::RBracket)?;
            return Ok(Type::Array(Box::new(inner)));
        }

        let token = self.advance().clone();
        match token.kind {
            TokenKind::TypeInt => Ok(Type::Int),
            TokenKind::TypeFloat => Ok(Type::Float),
            TokenKind::TypeBool => Ok(Type::Bool),
            TokenKind::TypeStr => Ok(Type::Str),
            TokenKind::TypeChar => Ok(Type::Char),
            TokenKind::TypeU8 => Ok(Type::U8),
            TokenKind::TypeVoid => Ok(Type::Void),
            TokenKind::Ident(name) => {
                if self.check(&TokenKind::Lt) {
                    self.advance();
                    let mut type_args = Vec::new();
                    while !self.check(&TokenKind::Gt) && !self.is_at_end() {
                        type_args.push(self.parse_type()?);
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::Gt)?;
                    if name == "Option" {
                        if type_args.len() != 1 {
                            return Err(format!("Hata [{}:{}]: 'Option' tam 1 tip parametresi bekler: Option<T>", token.span.line, token.span.col));
                        }
                        Ok(Type::Option(Box::new(type_args.remove(0))))
                    } else if name == "Result" {
                        if type_args.len() != 2 {
                            return Err(format!("Hata [{}:{}]: 'Result' tam 2 tip parametresi bekler: Result<T, E>", token.span.line, token.span.col));
                        }
                        let err_ty = type_args.remove(1);
                        let ok_ty = type_args.remove(0);
                        Ok(Type::Result(Box::new(ok_ty), Box::new(err_ty)))
                    } else if name == "Box" {
                        if type_args.len() != 1 {
                            return Err(format!("Hata [{}:{}]: 'Box' tam 1 tip parametresi bekler: Box<T>", token.span.line, token.span.col));
                        }
                        Ok(Type::Box(Box::new(type_args.remove(0))))
                    } else if name == "Rc" {
                        if type_args.len() != 1 {
                            return Err(format!("Hata [{}:{}]: 'Rc' tam 1 tip parametresi bekler: Rc<T>", token.span.line, token.span.col));
                        }
                        Ok(Type::Rc(Box::new(type_args.remove(0))))
                    } else if name == "RefCell" {
                        if type_args.len() != 1 {
                            return Err(format!("Hata [{}:{}]: 'RefCell' tam 1 tip parametresi bekler: RefCell<T>", token.span.line, token.span.col));
                        }
                        Ok(Type::RefCell(Box::new(type_args.remove(0))))
                    } else if name == "JoinHandle" {
                        if type_args.len() != 1 {
                            return Err(format!("Hata [{}:{}]: 'JoinHandle' tam 1 tip parametresi bekler: JoinHandle<T>", token.span.line, token.span.col));
                        }
                        Ok(Type::JoinHandle(Box::new(type_args.remove(0))))
                    } else if name == "Sender" {
                        if type_args.len() != 1 {
                            return Err(format!("Hata [{}:{}]: 'Sender' tam 1 tip parametresi bekler: Sender<T>", token.span.line, token.span.col));
                        }
                        Ok(Type::Sender(Box::new(type_args.remove(0))))
                    } else if name == "Receiver" {
                        if type_args.len() != 1 {
                            return Err(format!("Hata [{}:{}]: 'Receiver' tam 1 tip parametresi bekler: Receiver<T>", token.span.line, token.span.col));
                        }
                        Ok(Type::Receiver(Box::new(type_args.remove(0))))
                    } else if name == "Mutex" {
                        if type_args.len() != 1 {
                            return Err(format!("Hata [{}:{}]: 'Mutex' tam 1 tip parametresi bekler: Mutex<T>", token.span.line, token.span.col));
                        }
                        Ok(Type::Mutex(Box::new(type_args.remove(0))))
                    } else {
                        Ok(Type::Generic {
                            name: name.clone(),
                            type_args,
                        })
                    }
                } else if name == "TcpListener" {
                    Ok(Type::TcpListener)
                } else if name == "TcpStream" {
                    Ok(Type::TcpStream)
                } else {
                    Ok(Type::Named(name.clone()))
                }
            }
            _ => Err(format!(
                "Hata [{}:{}]: Geçersiz tip tanımlaması '{:?}'",
                token.span.line, token.span.col, token.kind
            )),
        }
    }

    pub fn parse_block(&mut self) -> Result<Block, String> {
        let lbrace = self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        let mut trailing_expr = None;

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            if self.check(&TokenKind::Let) {
                stmts.push(self.parse_let_stmt()?);
            } else if self.check(&TokenKind::Const) {
                stmts.push(Stmt::Const(self.parse_const()?));
            } else if self.check(&TokenKind::While) {
                stmts.push(self.parse_while_stmt()?);
            } else if self.check(&TokenKind::For) {
                stmts.push(self.parse_for_stmt()?);
            } else if self.check(&TokenKind::Break) {
                let span = self.advance().span;
                self.expect(TokenKind::Semicolon)?;
                stmts.push(Stmt::Break(span));
            } else if self.check(&TokenKind::Continue) {
                let span = self.advance().span;
                self.expect(TokenKind::Semicolon)?;
                stmts.push(Stmt::Continue(span));
            } else if self.check(&TokenKind::Return) {
                stmts.push(self.parse_return_stmt()?);
            } else {
                let expr = self.parse_expression(0)?;
                if self.check(&TokenKind::Semicolon) {
                    self.advance();
                    stmts.push(Stmt::Expr(expr));
                } else if self.check(&TokenKind::RBrace) {
                    // Rust-style trailing expression (block return value)
                    trailing_expr = Some(Box::new(expr));
                    break;
                } else {
                    // Control flow statements like 'if' and 'match' can omit semicolon
                    match expr {
                        Expr::If { .. } | Expr::Block(_) | Expr::Match { .. } | Expr::UnsafeBlock(..) => {
                            stmts.push(Stmt::Expr(expr));
                        }
                        _ => {
                            let span = self.peek().span;
                            return Err(format!(
                                "Hata [{}:{}]: İfadeden sonra ';' veya blok kapanışı '}}' bekleniyordu",
                                span.line, span.col
                            ));
                        }
                    }
                }
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Block {
            stmts,
            trailing_expr,
            span: lbrace.span,
        })
    }

    fn parse_const(&mut self) -> Result<ConstDef, String> {
        let const_tok = self.expect(TokenKind::Const)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Eq)?;
        let value = self.parse_expression(0)?;
        self.expect(TokenKind::Semicolon)?;
        Ok(ConstDef {
            name,
            ty,
            value,
            span: const_tok.span,
        })
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, String> {
        let let_token = self.expect(TokenKind::Let)?;
        let is_mut = if self.check(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        if self.check(&TokenKind::LParen) {
            self.advance();
            let mut elements = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                let elem_mut = if self.check(&TokenKind::Mut) {
                    self.advance();
                    true
                } else {
                    is_mut
                };
                let elem_name = self.expect_ident()?;
                elements.push((elem_name, elem_mut));
                if self.check(&TokenKind::Comma) {
                    self.advance();
                    if self.check(&TokenKind::RParen) {
                        break;
                    }
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;

            let ty = if self.check(&TokenKind::Colon) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };

            let init = if self.check(&TokenKind::Eq) {
                self.advance();
                Some(self.parse_expression(0)?)
            } else {
                None
            };

            self.expect(TokenKind::Semicolon)?;

            return Ok(Stmt::LetTuple {
                elements,
                ty,
                init,
                span: let_token.span,
            });
        }

        let name = self.expect_ident()?;

        let ty = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let init = if self.check(&TokenKind::Eq) {
            self.advance();
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::Let {
            name,
            is_mut,
            ty,
            init,
            span: let_token.span,
        })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, String> {
        let while_tok = self.expect(TokenKind::While)?;
        let condition = self.parse_expression(0)?;
        let body = self.parse_block()?;
        Ok(Stmt::While {
            condition,
            body,
            span: while_tok.span,
        })
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, String> {
        let for_tok = self.expect(TokenKind::For)?;
        let var_name = self.expect_ident()?;
        self.expect(TokenKind::In)?;
        let first_expr = self.parse_expression(0)?;
        if self.check(&TokenKind::DotDot) {
            self.advance();
            let end_expr = self.parse_expression(0)?;
            let body = self.parse_block()?;
            Ok(Stmt::ForRange {
                var_name,
                start: first_expr,
                end: end_expr,
                body,
                span: for_tok.span,
            })
        } else {
            let body = self.parse_block()?;
            Ok(Stmt::ForIn {
                var_name,
                iterable: first_expr,
                body,
                span: for_tok.span,
            })
        }
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, String> {
        let ret_tok = self.expect(TokenKind::Return)?;
        let val = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression(0)?)
        };
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Return(val, ret_tok.span))
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        let tok = self.peek().clone();
        if let TokenKind::Ident(s) = tok.kind {
            self.advance();
            Ok(s)
        } else {
            Err(format!(
                "Hata [{}:{}]: İsim (identifier) bekleniyordu, bulunan: {:?}",
                tok.span.line, tok.span.col, tok.kind
            ))
        }
    }

    // Pratt Parser for expressions
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, String> {
        let mut lhs = self.parse_prefix()?;

        loop {
            if self.is_at_end() {
                break;
            }

            // Postfix array/slice indexing or slicing: arr[index], arr[start..end], arr[start..], arr[..end], arr[..]
            if self.check(&TokenKind::LBracket) {
                let span = self.peek().span;
                self.advance();
                if self.check(&TokenKind::DotDot) {
                    self.advance();
                    let end = if self.check(&TokenKind::RBracket) {
                        None
                    } else {
                        Some(Box::new(self.parse_expression(0)?))
                    };
                    self.expect(TokenKind::RBracket)?;
                    lhs = Expr::Slice {
                        target: Box::new(lhs),
                        start: None,
                        end,
                        span,
                    };
                } else {
                    let first_expr = self.parse_expression(0)?;
                    if self.check(&TokenKind::DotDot) {
                        self.advance();
                        let end = if self.check(&TokenKind::RBracket) {
                            None
                        } else {
                            Some(Box::new(self.parse_expression(0)?))
                        };
                        self.expect(TokenKind::RBracket)?;
                        lhs = Expr::Slice {
                            target: Box::new(lhs),
                            start: Some(Box::new(first_expr)),
                            end,
                            span,
                        };
                    } else {
                        self.expect(TokenKind::RBracket)?;
                        lhs = Expr::Index {
                            target: Box::new(lhs),
                            index: Box::new(first_expr),
                            span,
                        };
                    }
                }
                continue;
            }

            // Postfix try operator: expr?
            if self.check(&TokenKind::Question) {
                let q_span = self.advance().span;
                lhs = Expr::Try(Box::new(lhs), q_span);
                continue;
            }

            // Postfix dot access: lhs.field or lhs.method(args...) or lhs.0 (tuple index)
            if self.check(&TokenKind::Dot) {
                let dot_span = self.peek().span;
                self.advance();
                if let TokenKind::IntLit(idx) = self.peek().kind {
                    let idx_val = idx as usize;
                    self.advance();
                    lhs = Expr::TupleIndex {
                        target: Box::new(lhs),
                        index: idx_val,
                        span: dot_span,
                    };
                    continue;
                }
                let member = self.expect_ident()?;
                let mut type_args = Vec::new();
                if self.check(&TokenKind::ColonColon) && self.pos + 1 < self.tokens.len() && self.tokens[self.pos + 1].kind == TokenKind::Lt {
                    self.advance();
                    self.advance();
                    while !self.check(&TokenKind::Gt) && !self.is_at_end() {
                        type_args.push(self.parse_type()?);
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::Gt)?;
                }
                if self.check(&TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expression(0)?);
                            if self.check(&TokenKind::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    lhs = Expr::MethodCall {
                        target: Box::new(lhs),
                        method: member,
                        type_args,
                        args,
                        span: dot_span,
                    };
                } else {
                    lhs = Expr::FieldAccess {
                        target: Box::new(lhs),
                        field: member,
                        span: dot_span,
                    };
                }
                continue;
            }

            // Check assignment: ident = expr OR arr[i] = expr OR p.x = expr
            if self.check(&TokenKind::Eq) {
                if min_bp <= 1 {
                    let eq_span = self.peek().span;
                    self.advance();
                    let rhs = self.parse_expression(1)?;
                    match lhs {
                        Expr::Ident(name, _) => {
                            lhs = Expr::Assign {
                                target: name,
                                value: Box::new(rhs),
                                span: eq_span,
                            };
                            continue;
                        }
                        Expr::Index { target, index, span } => {
                            lhs = Expr::IndexAssign {
                                target,
                                index,
                                value: Box::new(rhs),
                                span,
                            };
                            continue;
                        }
                        Expr::FieldAccess { target, field, span } => {
                            lhs = Expr::FieldAssign {
                                target,
                                field,
                                value: Box::new(rhs),
                                span,
                            };
                            continue;
                        }
                        Expr::TupleIndex { target, index, span } => {
                            lhs = Expr::TupleIndexAssign {
                                target,
                                index,
                                value: Box::new(rhs),
                                span,
                            };
                            continue;
                        }
                        Expr::Deref { expr, span: _ } => {
                            lhs = Expr::DerefAssign {
                                target: expr,
                                value: Box::new(rhs),
                                span: eq_span,
                            };
                            continue;
                        }
                        _ => return Err(format!("Hata [{}:{}]: Atama yalnızca değişkenlere, alanlara veya dizi indekslerine yapılabilir", eq_span.line, eq_span.col)),
                    }
                } else {
                    break;
                }
            }

            let op_info = match self.peek_kind() {
                TokenKind::OrOr => Some((BinOp::Or, 2, 3)),
                TokenKind::AndAnd => Some((BinOp::And, 4, 5)),
                TokenKind::EqEq => Some((BinOp::Eq, 6, 7)),
                TokenKind::BangEq => Some((BinOp::NotEq, 6, 7)),
                TokenKind::Lt => Some((BinOp::Lt, 8, 9)),
                TokenKind::LtEq => Some((BinOp::LtEq, 8, 9)),
                TokenKind::Gt => Some((BinOp::Gt, 8, 9)),
                TokenKind::GtEq => Some((BinOp::GtEq, 8, 9)),
                TokenKind::Plus => Some((BinOp::Add, 10, 11)),
                TokenKind::Minus => Some((BinOp::Sub, 10, 11)),
                TokenKind::Star => Some((BinOp::Mul, 12, 13)),
                TokenKind::Slash => Some((BinOp::Div, 12, 13)),
                TokenKind::Percent => Some((BinOp::Mod, 12, 13)),
                _ => None,
            };

            if let Some((op, l_bp, r_bp)) = op_info {
                if l_bp < min_bp {
                    break;
                }
                let op_tok = self.advance();
                let span = op_tok.span;
                let rhs = self.parse_expression(r_bp)?;
                lhs = Expr::Binary {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                    span,
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    fn parse_format_string(&mut self, raw: &str, span: Span) -> Result<Expr, String> {
        let mut parts = Vec::new();
        let mut current_lit = String::new();
        let mut chars = raw.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    current_lit.push('{');
                } else {
                    if !current_lit.is_empty() {
                        parts.push(FormatPart::Lit(std::mem::take(&mut current_lit)));
                    }
                    let mut expr_text = String::new();
                    let mut depth = 1;
                    while let Some(ec) = chars.next() {
                        if ec == '{' {
                            depth += 1;
                            expr_text.push(ec);
                        } else if ec == '}' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            expr_text.push(ec);
                        } else {
                            expr_text.push(ec);
                        }
                    }
                    if depth != 0 {
                        return Err(format!("Hata [{}:{}]: Format metninde kapanmamış '}}' bulundu", span.line, span.col));
                    }
                    let trimmed = expr_text.trim();
                    if trimmed.is_empty() {
                        return Err(format!("Hata [{}:{}]: Format metninde boş '{{}}' ifadesi bulundu", span.line, span.col));
                    }
                    let expr_tokens = crate::lexer::Lexer::new(trimmed).tokenize()?;
                    let mut expr_parser = Parser::new(expr_tokens);
                    let parsed_expr = expr_parser.parse_expression(0)?;
                    parts.push(FormatPart::Expr(Box::new(parsed_expr)));
                }
            } else if c == '}' {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    current_lit.push('}');
                } else {
                    return Err(format!("Hata [{}:{}]: Format metninde eşleşmeyen tek '}}' bulundu ('}}}}' kullanın)", span.line, span.col));
                }
            } else {
                current_lit.push(c);
            }
        }
        if !current_lit.is_empty() {
            parts.push(FormatPart::Lit(current_lit));
        }
        Ok(Expr::FormatString(parts, span))
    }

    fn desugar_format_call(&mut self, template: &str, args: Vec<Expr>, span: Span) -> Result<Expr, String> {
        let mut parts = Vec::new();
        let mut current_lit = String::new();
        let mut arg_iter = args.into_iter();
        let mut chars = template.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    current_lit.push('{');
                } else if chars.peek() == Some(&'}') {
                    chars.next();
                    if !current_lit.is_empty() {
                        parts.push(FormatPart::Lit(std::mem::take(&mut current_lit)));
                    }
                    let next_arg = arg_iter.next().ok_or_else(|| {
                        format!("Hata [{}:{}]: 'format()' fonksiyonunda '{{}}' için yeterli argüman verilmedi", span.line, span.col)
                    })?;
                    parts.push(FormatPart::Expr(Box::new(next_arg)));
                } else {
                    return Err(format!("Hata [{}:{}]: 'format()' şablonunda '{{}}' veya '{{{{' bekleniyordu", span.line, span.col));
                }
            } else if c == '}' {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    current_lit.push('}');
                } else {
                    return Err(format!("Hata [{}:{}]: 'format()' şablonunda eşleşmeyen tek '}}' bulundu ('}}}}' kullanın)", span.line, span.col));
                }
            } else {
                current_lit.push(c);
            }
        }
        if arg_iter.next().is_some() {
            return Err(format!("Hata [{}:{}]: 'format()' fonksiyonuna şablondan daha fazla argüman verildi", span.line, span.col));
        }
        if !current_lit.is_empty() {
            parts.push(FormatPart::Lit(current_lit));
        }
        Ok(Expr::FormatString(parts, span))
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::IntLit(val) => {
                self.advance();
                Ok(Expr::Int(val, tok.span))
            }
            TokenKind::FloatLit(val) => {
                self.advance();
                Ok(Expr::Float(val, tok.span))
            }
            TokenKind::BoolLit(val) => {
                self.advance();
                Ok(Expr::Bool(val, tok.span))
            }
            TokenKind::StringLit(val) => {
                self.advance();
                Ok(Expr::Str(val, tok.span))
            }
            TokenKind::FormatString(val) => {
                self.advance();
                self.parse_format_string(&val, tok.span)
            }
            TokenKind::CharLit(val) => {
                self.advance();
                Ok(Expr::Char(val, tok.span))
            }
            TokenKind::U8Lit(val) => {
                self.advance();
                Ok(Expr::U8(val, tok.span))
            }
            TokenKind::Ident(name) => {
                self.advance();
                if self.check(&TokenKind::ColonColon) {
                    if self.pos + 1 < self.tokens.len() && self.tokens[self.pos + 1].kind == TokenKind::Lt {
                        self.advance(); // consume ::
                        self.advance(); // consume <
                        let mut type_args = Vec::new();
                        while !self.check(&TokenKind::Gt) && !self.is_at_end() {
                            type_args.push(self.parse_type()?);
                            if self.check(&TokenKind::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        self.expect(TokenKind::Gt)?;
                        if self.check(&TokenKind::LParen) {
                            self.advance();
                            let mut args = Vec::new();
                            if !self.check(&TokenKind::RParen) {
                                loop {
                                    args.push(self.parse_expression(0)?);
                                    if self.check(&TokenKind::Comma) {
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                            }
                            self.expect(TokenKind::RParen)?;
                            Ok(Expr::Call {
                                callee: name,
                                type_args,
                                args,
                                span: tok.span,
                            })
                        } else if self.check(&TokenKind::LBrace) {
                            self.advance();
                            let mut fields = Vec::new();
                            while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                                let field_name = self.expect_ident()?;
                                self.expect(TokenKind::Colon)?;
                                let field_val = self.parse_expression(0)?;
                                fields.push((field_name, field_val));
                                if self.check(&TokenKind::Comma) {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            self.expect(TokenKind::RBrace)?;
                            Ok(Expr::StructInit {
                                name,
                                type_args,
                                fields,
                                span: tok.span,
                            })
                        } else {
                            Err(format!("Hata [{}:{}]: '::' sonrasında generic çağrı veya struct başlatıcı bekleniyordu", tok.span.line, tok.span.col))
                        }
                    } else {
                        self.advance();
                        let member = self.expect_ident()?;
                        if self.check(&TokenKind::LParen) {
                            self.advance();
                            let mut args = Vec::new();
                            if !self.check(&TokenKind::RParen) {
                                loop {
                                    args.push(self.parse_expression(0)?);
                                    if self.check(&TokenKind::Comma) {
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                            }
                            self.expect(TokenKind::RParen)?;
                            Ok(Expr::Call {
                                callee: format!("{}::{}", name, member),
                                type_args: Vec::new(),
                                args,
                                span: tok.span,
                            })
                        } else {
                            Ok(Expr::Path {
                                base: name,
                                member,
                                span: tok.span,
                            })
                        }
                    }
                } else if self.check(&TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expression(0)?);
                            if self.check(&TokenKind::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    if name == "format" && !args.is_empty() {
                        if let Expr::Str(template, s_span) = &args[0] {
                            let template_str = template.clone();
                            let s_sp = *s_span;
                            let remaining_args = args.into_iter().skip(1).collect();
                            return self.desugar_format_call(&template_str, remaining_args, s_sp);
                        }
                    }
                    Ok(Expr::Call {
                        callee: name,
                        type_args: Vec::new(),
                        args,
                        span: tok.span,
                    })
                } else if self.check(&TokenKind::LBrace) && {
                    if self.pos + 1 < self.tokens.len() {
                        match &self.tokens[self.pos + 1].kind {
                            TokenKind::RBrace => true,
                            TokenKind::Ident(_) => {
                                self.pos + 2 < self.tokens.len() && matches!(self.tokens[self.pos + 2].kind, TokenKind::Colon)
                            }
                            _ => false,
                        }
                    } else {
                        false
                    }
                } {
                    self.advance(); // consume LBrace
                    let mut fields = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                        let field_name = self.expect_ident()?;
                        self.expect(TokenKind::Colon)?;
                        let field_val = self.parse_expression(0)?;
                        fields.push((field_name, field_val));
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    Ok(Expr::StructInit {
                        name,
                        type_args: Vec::new(),
                        fields,
                        span: tok.span,
                    })
                } else {
                    Ok(Expr::Ident(name, tok.span))
                }
            }
            TokenKind::Minus => {
                self.advance();
                let operand = self.parse_expression(14)?;
                Ok(Expr::Unary {
                    op: UnOp::Neg,
                    expr: Box::new(operand),
                    span: tok.span,
                })
            }
            TokenKind::Bang => {
                self.advance();
                let operand = self.parse_expression(14)?;
                Ok(Expr::Unary {
                    op: UnOp::Not,
                    expr: Box::new(operand),
                    span: tok.span,
                })
            }
            TokenKind::Ampersand => {
                let span = self.advance().span;
                let is_mut = if self.check(&TokenKind::Mut) {
                    self.advance();
                    true
                } else {
                    false
                };
                let operand = self.parse_expression(14)?;
                Ok(Expr::Borrow {
                    expr: Box::new(operand),
                    is_mut,
                    span,
                })
            }
            TokenKind::Star => {
                let span = self.advance().span;
                let operand = self.parse_expression(14)?;
                Ok(Expr::Deref {
                    expr: Box::new(operand),
                    span,
                })
            }
            TokenKind::Unsafe => {
                let span = self.advance().span;
                let block = self.parse_block()?;
                Ok(Expr::UnsafeBlock(block, span))
            }
            TokenKind::LParen => {
                let span = self.advance().span;
                if self.check(&TokenKind::RParen) {
                    self.advance();
                    return Ok(Expr::Tuple(Vec::new(), span));
                }
                let first = self.parse_expression(0)?;
                if self.check(&TokenKind::Comma) {
                    self.advance();
                    let mut elements = vec![first];
                    while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                        elements.push(self.parse_expression(0)?);
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    Ok(Expr::Tuple(elements, span))
                } else {
                    self.expect(TokenKind::RParen)?;
                    Ok(first)
                }
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        elements.push(self.parse_expression(0)?);
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Expr::Array(elements, tok.span))
            }
            TokenKind::Pipe | TokenKind::OrOr => {
                let span = self.advance().span;
                let mut params = Vec::new();

                if tok.kind == TokenKind::Pipe {
                    while !self.check(&TokenKind::Pipe) && !self.is_at_end() {
                        let is_mut = if self.check(&TokenKind::Mut) {
                            self.advance();
                            true
                        } else {
                            false
                        };
                        let name = self.expect_ident()?;
                        let ty = if self.check(&TokenKind::Colon) {
                            self.advance();
                            self.parse_type()?
                        } else {
                            Type::Int
                        };
                        params.push(Param { name, ty, is_mut });

                        if self.check(&TokenKind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect(TokenKind::Pipe)?;
                }

                let return_type = if self.check(&TokenKind::Arrow) {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };

                let body = if self.check(&TokenKind::LBrace) {
                    self.parse_block()?
                } else {
                    let expr = self.parse_expression(0)?;
                    let e_span = expr.span();
                    Block {
                        stmts: vec![],
                        trailing_expr: Some(Box::new(expr)),
                        span: e_span,
                    }
                };

                Ok(Expr::Closure {
                    params,
                    return_type,
                    body,
                    span,
                })
            }
            TokenKind::LBrace => {
                let block = self.parse_block()?;
                Ok(Expr::Block(block))
            }
            TokenKind::If => {
                self.advance();
                let condition = self.parse_expression(0)?;
                let then_branch = self.parse_block()?;
                let else_branch = if self.check(&TokenKind::Else) {
                    self.advance();
                    if self.check(&TokenKind::If) {
                        // else if -> nest as an if expression inside a block
                        let nested_if = self.parse_prefix()?;
                        Some(Block {
                            stmts: vec![],
                            trailing_expr: Some(Box::new(nested_if)),
                            span: tok.span,
                        })
                    } else {
                        Some(self.parse_block()?)
                    }
                } else {
                    None
                };

                Ok(Expr::If {
                    condition: Box::new(condition),
                    then_branch,
                    else_branch,
                    span: tok.span,
                })
            }
            TokenKind::Match => {
                self.advance();
                let subject = self.parse_expression(0)?;
                self.expect(TokenKind::LBrace)?;

                let mut arms = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                    let pattern = self.parse_pattern()?;
                    self.expect(TokenKind::FatArrow)?;
                    let body = self.parse_expression(0)?;
                    arms.push(MatchArm {
                        pattern,
                        body,
                        span: tok.span,
                    });

                    if self.check(&TokenKind::Comma) {
                        self.advance();
                    }
                }

                self.expect(TokenKind::RBrace)?;
                Ok(Expr::Match {
                    subject: Box::new(subject),
                    arms,
                    span: tok.span,
                })
            }
            _ => Err(format!(
                "Hata [{}:{}]: Beklenmeyen token '{:?}' ifade başlangıcında",
                tok.span.line, tok.span.col, tok.kind
            )),
        }
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::IntLit(i) => {
                self.advance();
                Ok(Pattern::Int(i, tok.span))
            }
            TokenKind::FloatLit(f) => {
                self.advance();
                Ok(Pattern::Float(f, tok.span))
            }
            TokenKind::BoolLit(b) => {
                self.advance();
                Ok(Pattern::Bool(b, tok.span))
            }
            TokenKind::StringLit(s) => {
                self.advance();
                Ok(Pattern::Str(s, tok.span))
            }
            TokenKind::CharLit(c) => {
                self.advance();
                Ok(Pattern::Char(c, tok.span))
            }
            TokenKind::U8Lit(b) => {
                self.advance();
                Ok(Pattern::U8(b, tok.span))
            }
            TokenKind::Ident(name) => {
                self.advance();
                if name == "_" {
                    Ok(Pattern::Wildcard(tok.span))
                } else if name == "Some" && self.check(&TokenKind::LParen) {
                    self.advance();
                    let inner = self.parse_pattern()?;
                    self.expect(TokenKind::RParen)?;
                    Ok(Pattern::Some(Box::new(inner), tok.span))
                } else if name == "None" {
                    Ok(Pattern::None(tok.span))
                } else if name == "Ok" && self.check(&TokenKind::LParen) {
                    self.advance();
                    let inner = self.parse_pattern()?;
                    self.expect(TokenKind::RParen)?;
                    Ok(Pattern::Ok(Box::new(inner), tok.span))
                } else if name == "Err" && self.check(&TokenKind::LParen) {
                    self.advance();
                    let inner = self.parse_pattern()?;
                    self.expect(TokenKind::RParen)?;
                    Ok(Pattern::Err(Box::new(inner), tok.span))
                } else if self.check(&TokenKind::ColonColon) {
                    self.advance();
                    let variant = self.expect_ident()?;
                    if name == "Option" {
                        if variant == "Some" && self.check(&TokenKind::LParen) {
                            self.advance();
                            let inner = self.parse_pattern()?;
                            self.expect(TokenKind::RParen)?;
                            return Ok(Pattern::Some(Box::new(inner), tok.span));
                        } else if variant == "None" {
                            return Ok(Pattern::None(tok.span));
                        }
                    } else if name == "Result" {
                        if variant == "Ok" && self.check(&TokenKind::LParen) {
                            self.advance();
                            let inner = self.parse_pattern()?;
                            self.expect(TokenKind::RParen)?;
                            return Ok(Pattern::Ok(Box::new(inner), tok.span));
                        } else if variant == "Err" && self.check(&TokenKind::LParen) {
                            self.advance();
                            let inner = self.parse_pattern()?;
                            self.expect(TokenKind::RParen)?;
                            return Ok(Pattern::Err(Box::new(inner), tok.span));
                        }
                    }
                    Ok(Pattern::EnumVariant {
                        enum_name: name,
                        variant,
                        span: tok.span,
                    })
                } else {
                    Ok(Pattern::Ident(name, tok.span))
                }
            }
            TokenKind::LParen => {
                let span = self.advance().span;
                let mut pats = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    loop {
                        pats.push(self.parse_pattern()?);
                        if self.check(&TokenKind::Comma) {
                            self.advance();
                            if self.check(&TokenKind::RParen) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RParen)?;
                Ok(Pattern::Tuple(pats, span))
            }
            _ => Err(format!(
                "Hata [{}:{}]: Geçersiz match deseni (pattern): '{:?}'",
                tok.span.line, tok.span.col, tok.kind
            )),
        }
    }
}
