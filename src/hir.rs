use std::fmt;
use crate::ast::{BinOp, Block, Expr, FormatPart, Function, Item, Program, Stmt, Type, UnOp};
use crate::eval::Value;
use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockId(pub usize);

impl fmt::Display for BasicBlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Place {
    Local(String),
    Field {
        base: Box<Place>,
        field: String,
    },
    Index {
        base: Box<Place>,
        index: Box<Operand>,
    },
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Place::Local(name) => write!(f, "{}", name),
            Place::Field { base, field } => write!(f, "{}.{}", base, field),
            Place::Index { base, index } => write!(f, "{}[{}]", base, index),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Copy(Place),
    Constant(Value),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Copy(p) => write!(f, "{}", p),
            Operand::Constant(v) => match v {
                Value::Str(s) => write!(f, "\"{}\"", s),
                Value::Char(c) => write!(f, "'{}'", c),
                other => write!(f, "{}", other),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Rvalue {
    Use(Operand),
    BinaryOp(BinOp, Operand, Operand),
    UnaryOp(UnOp, Operand),
    Array(Vec<Operand>),
    Struct(String, Vec<(String, Operand)>),
    #[allow(dead_code)]
    Ref {
        is_mut: bool,
        place: Place,
    },
}

impl fmt::Display for Rvalue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rvalue::Use(op) => write!(f, "{}", op),
            Rvalue::BinaryOp(op, l, r) => write!(f, "{} {:?} {}", l, op, r),
            Rvalue::UnaryOp(op, inner) => write!(f, "{:?}{}", op, inner),
            Rvalue::Array(elems) => {
                let s = elems.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "[{}]", s)
            }
            Rvalue::Struct(name, fields) => {
                let f_strs = fields.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ");
                write!(f, "{} {{ {} }}", name, f_strs)
            }
            Rvalue::Ref { is_mut, place } => {
                if *is_mut {
                    write!(f, "&mut {}", place)
                } else {
                    write!(f, "&{}", place)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStatement {
    Assign {
        place: Place,
        rvalue: Rvalue,
        span: Span,
    },
    Call {
        dest: Option<Place>,
        callee: String,
        args: Vec<Operand>,
        span: Span,
    },
}

impl fmt::Display for HirStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HirStatement::Assign { place, rvalue, .. } => {
                write!(f, "{} = {}", place, rvalue)
            }
            HirStatement::Call { dest, callee, args, .. } => {
                let args_s = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                if let Some(d) = dest {
                    write!(f, "{} = call {}({})", d, callee, args_s)
                } else {
                    write!(f, "call {}({})", callee, args_s)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    Jump(BasicBlockId, Span),
    Branch {
        cond: Operand,
        then_block: BasicBlockId,
        else_block: BasicBlockId,
        span: Span,
    },
    Return(Option<Operand>, Span),
    Unreachable,
}

impl fmt::Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminator::Jump(target, _) => write!(f, "jump -> {}", target),
            Terminator::Branch { cond, then_block, else_block, .. } => {
                write!(f, "branch {} -> {}, else -> {}", cond, then_block, else_block)
            }
            Terminator::Return(Some(op), _) => write!(f, "return {}", op),
            Terminator::Return(None, _) => write!(f, "return"),
            Terminator::Unreachable => write!(f, "unreachable"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub statements: Vec<HirStatement>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret_type: Type,
    pub blocks: Vec<BasicBlock>,
    pub start_block: BasicBlockId,
}

impl fmt::Display for HirFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params_s = self
            .params
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, ty))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(f, "fn {}({}) -> {} {{", self.name, params_s, self.ret_type)?;
        for bb in &self.blocks {
            writeln!(f, "  {}:", bb.id)?;
            for stmt in &bb.statements {
                writeln!(f, "    {}", stmt)?;
            }
            writeln!(f, "    {}", bb.terminator)?;
        }
        writeln!(f, "}}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirProgram {
    pub functions: Vec<HirFunction>,
}

impl fmt::Display for HirProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "/* ======================================= */")?;
        writeln!(f, "/* Oxide High-Level Intermediate Rep (HIR) */")?;
        writeln!(f, "/* Control Flow Graphs (CFG & BasicBlocks) */")?;
        writeln!(f, "/* ======================================= */\n")?;
        for (i, func) in self.functions.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", func)?;
        }
        Ok(())
    }
}

/// AST to HIR Lowering Builder
pub struct HirBuilder {
    temp_counter: usize,
    blocks: Vec<BasicBlock>,
    current_bb: BasicBlockId,
    loop_stack: Vec<(BasicBlockId, BasicBlockId)>, // (continue_target, break_target)
}

impl HirBuilder {
    pub fn new() -> Self {
        let entry = BasicBlock {
            id: BasicBlockId(0),
            statements: Vec::new(),
            terminator: Terminator::Unreachable,
        };
        Self {
            temp_counter: 0,
            blocks: vec![entry],
            current_bb: BasicBlockId(0),
            loop_stack: Vec::new(),
        }
    }

    fn new_temp(&mut self) -> String {
        let id = self.temp_counter;
        self.temp_counter += 1;
        format!("_t{}", id)
    }

    fn new_block(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.blocks.len());
        self.blocks.push(BasicBlock {
            id,
            statements: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        id
    }

    fn emit_stmt(&mut self, stmt: HirStatement) {
        self.blocks[self.current_bb.0].statements.push(stmt);
    }

    fn terminate(&mut self, term: Terminator) {
        self.blocks[self.current_bb.0].terminator = term;
    }

    fn is_current_terminated(&self) -> bool {
        self.blocks[self.current_bb.0].terminator != Terminator::Unreachable
    }

    pub fn lower_program(program: &Program) -> HirProgram {
        let mut hir_funcs = Vec::new();
        for item in &program.items {
            match item {
                Item::Function(f) => {
                    let mut builder = HirBuilder::new();
                    let hir_f = builder.lower_function(f);
                    hir_funcs.push(hir_f);
                }
                Item::Impl(imp) => {
                    for m in &imp.methods {
                        let mut builder = HirBuilder::new();
                        let mut m_f = m.clone();
                        m_f.name = format!("{}_{}", imp.target, m.name);
                        let hir_f = builder.lower_function(&m_f);
                        hir_funcs.push(hir_f);
                    }
                }
                _ => {}
            }
        }
        HirProgram { functions: hir_funcs }
    }

    pub fn lower_function(&mut self, f: &Function) -> HirFunction {
        self.lower_block(&f.body);

        if !self.is_current_terminated() {
            let span = f.span;
            if let Some(trailing) = &f.body.trailing_expr {
                let op = self.lower_expr(trailing);
                self.terminate(Terminator::Return(Some(op), span));
            } else if f.return_type == Type::Void {
                self.terminate(Terminator::Return(None, span));
            } else {
                self.terminate(Terminator::Return(None, span));
            }
        }

        HirFunction {
            name: f.name.clone(),
            params: f.params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
            ret_type: f.return_type.clone(),
            blocks: std::mem::take(&mut self.blocks),
            start_block: BasicBlockId(0),
        }
    }

    fn lower_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            if self.is_current_terminated() {
                break;
            }
            self.lower_stmt(stmt);
        }
    }

    fn lower_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, init, span, .. } => {
                if let Some(expr) = init {
                    let op = self.lower_expr(expr);
                    self.emit_stmt(HirStatement::Assign {
                        place: Place::Local(name.clone()),
                        rvalue: Rvalue::Use(op),
                        span: *span,
                    });
                }
            }
            Stmt::LetTuple { elements, init, span, .. } => {
                if let Some(expr) = init {
                    let tuple_op = self.lower_expr(expr);
                    let tmp = self.new_temp();
                    self.emit_stmt(HirStatement::Assign {
                        place: Place::Local(tmp.clone()),
                        rvalue: Rvalue::Use(tuple_op),
                        span: *span,
                    });
                    for (i, (name, _)) in elements.iter().enumerate() {
                        if name != "_" {
                            self.emit_stmt(HirStatement::Assign {
                                place: Place::Local(name.clone()),
                                rvalue: Rvalue::Use(Operand::Copy(Place::Field {
                                    base: Box::new(Place::Local(tmp.clone())),
                                    field: format!("_{}", i),
                                })),
                                span: *span,
                            });
                        }
                    }
                }
            }
            Stmt::Expr(expr) => {
                let _ = self.lower_expr(expr);
            }
            Stmt::Const(c) => {
                let op = self.lower_expr(&c.value);
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(c.name.clone()),
                    rvalue: Rvalue::Use(op),
                    span: c.span,
                });
            }
            Stmt::While { condition, body, span } => {
                let header_bb = self.new_block();
                let body_bb = self.new_block();
                let exit_bb = self.new_block();

                self.terminate(Terminator::Jump(header_bb, *span));
                self.current_bb = header_bb;

                let cond_op = self.lower_expr(condition);
                self.terminate(Terminator::Branch {
                    cond: cond_op,
                    then_block: body_bb,
                    else_block: exit_bb,
                    span: *span,
                });

                self.current_bb = body_bb;
                self.loop_stack.push((header_bb, exit_bb));
                self.lower_block(body);
                self.loop_stack.pop();

                if !self.is_current_terminated() {
                    self.terminate(Terminator::Jump(header_bb, *span));
                }

                self.current_bb = exit_bb;
            }
            Stmt::ForRange { var_name, start, end, body, span } => {
                let start_op = self.lower_expr(start);
                let end_op = self.lower_expr(end);

                let iter_var = Place::Local(var_name.clone());
                self.emit_stmt(HirStatement::Assign {
                    place: iter_var.clone(),
                    rvalue: Rvalue::Use(start_op),
                    span: *span,
                });

                let header_bb = self.new_block();
                let body_bb = self.new_block();
                let step_bb = self.new_block();
                let exit_bb = self.new_block();

                self.terminate(Terminator::Jump(header_bb, *span));
                self.current_bb = header_bb;

                let cond_temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(cond_temp.clone()),
                    rvalue: Rvalue::BinaryOp(BinOp::Lt, Operand::Copy(iter_var.clone()), end_op),
                    span: *span,
                });
                self.terminate(Terminator::Branch {
                    cond: Operand::Copy(Place::Local(cond_temp)),
                    then_block: body_bb,
                    else_block: exit_bb,
                    span: *span,
                });

                self.current_bb = body_bb;
                self.loop_stack.push((step_bb, exit_bb));
                self.lower_block(body);
                self.loop_stack.pop();

                if !self.is_current_terminated() {
                    self.terminate(Terminator::Jump(step_bb, *span));
                }

                self.current_bb = step_bb;
                let inc_temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(inc_temp.clone()),
                    rvalue: Rvalue::BinaryOp(BinOp::Add, Operand::Copy(iter_var.clone()), Operand::Constant(Value::Int(1))),
                    span: *span,
                });
                self.emit_stmt(HirStatement::Assign {
                    place: iter_var,
                    rvalue: Rvalue::Use(Operand::Copy(Place::Local(inc_temp))),
                    span: *span,
                });
                self.terminate(Terminator::Jump(header_bb, *span));

                self.current_bb = exit_bb;
            }
            Stmt::ForIn { var_name, iterable, body, span } => {
                // Lower array/str iteration into index-based loop
                let arr_op = self.lower_expr(iterable);
                let arr_temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(arr_temp.clone()),
                    rvalue: Rvalue::Use(arr_op),
                    span: *span,
                });

                let idx_temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(idx_temp.clone()),
                    rvalue: Rvalue::Use(Operand::Constant(Value::Int(0))),
                    span: *span,
                });

                let len_temp = self.new_temp();
                self.emit_stmt(HirStatement::Call {
                    dest: Some(Place::Local(len_temp.clone())),
                    callee: "len".to_string(),
                    args: vec![Operand::Copy(Place::Local(arr_temp.clone()))],
                    span: *span,
                });

                let header_bb = self.new_block();
                let body_bb = self.new_block();
                let step_bb = self.new_block();
                let exit_bb = self.new_block();

                self.terminate(Terminator::Jump(header_bb, *span));
                self.current_bb = header_bb;

                let cond_temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(cond_temp.clone()),
                    rvalue: Rvalue::BinaryOp(
                        BinOp::Lt,
                        Operand::Copy(Place::Local(idx_temp.clone())),
                        Operand::Copy(Place::Local(len_temp)),
                    ),
                    span: *span,
                });
                self.terminate(Terminator::Branch {
                    cond: Operand::Copy(Place::Local(cond_temp)),
                    then_block: body_bb,
                    else_block: exit_bb,
                    span: *span,
                });

                self.current_bb = body_bb;
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(var_name.clone()),
                    rvalue: Rvalue::Use(Operand::Copy(Place::Index {
                        base: Box::new(Place::Local(arr_temp)),
                        index: Box::new(Operand::Copy(Place::Local(idx_temp.clone()))),
                    })),
                    span: *span,
                });

                self.loop_stack.push((step_bb, exit_bb));
                self.lower_block(body);
                self.loop_stack.pop();

                if !self.is_current_terminated() {
                    self.terminate(Terminator::Jump(step_bb, *span));
                }

                self.current_bb = step_bb;
                let inc_temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(inc_temp.clone()),
                    rvalue: Rvalue::BinaryOp(
                        BinOp::Add,
                        Operand::Copy(Place::Local(idx_temp.clone())),
                        Operand::Constant(Value::Int(1)),
                    ),
                    span: *span,
                });
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(idx_temp),
                    rvalue: Rvalue::Use(Operand::Copy(Place::Local(inc_temp))),
                    span: *span,
                });
                self.terminate(Terminator::Jump(header_bb, *span));

                self.current_bb = exit_bb;
            }
            Stmt::Break(span) => {
                if let Some((_, break_target)) = self.loop_stack.last() {
                    let target = *break_target;
                    self.terminate(Terminator::Jump(target, *span));
                }
            }
            Stmt::Continue(span) => {
                if let Some((continue_target, _)) = self.loop_stack.last() {
                    let target = *continue_target;
                    self.terminate(Terminator::Jump(target, *span));
                }
            }
            Stmt::Return(expr, span) => {
                let op = expr.as_ref().map(|e| self.lower_expr(e));
                self.terminate(Terminator::Return(op, *span));
            }
        }
    }

    fn lower_expr(&mut self, expr: &Expr) -> Operand {
        match expr {
            Expr::Int(i, _) => Operand::Constant(Value::Int(*i)),
            Expr::Float(f, _) => Operand::Constant(Value::Float(*f)),
            Expr::Bool(b, _) => Operand::Constant(Value::Bool(*b)),
            Expr::Str(s, _) => Operand::Constant(Value::Str(s.clone())),
            Expr::Char(c, _) => Operand::Constant(Value::Char(*c)),
            Expr::U8(b, _) => Operand::Constant(Value::U8(*b)),
            Expr::Ident(name, _) => Operand::Copy(Place::Local(name.clone())),
            Expr::Unary { op, expr, span } => {
                let inner = self.lower_expr(expr);
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::UnaryOp(*op, inner),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::Binary { op, left, right, span } => {
                let l = self.lower_expr(left);
                let r = self.lower_expr(right);
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::BinaryOp(*op, l, r),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::Assign { target, value, span } => {
                let v = self.lower_expr(value);
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(target.clone()),
                    rvalue: Rvalue::Use(v.clone()),
                    span: *span,
                });
                v
            }
            Expr::Call { callee, args, span, .. } => {
                let mut lowered_args = Vec::new();
                for a in args {
                    lowered_args.push(self.lower_expr(a));
                }
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Call {
                    dest: Some(Place::Local(temp.clone())),
                    callee: callee.clone(),
                    args: lowered_args,
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::If { condition, then_branch, else_branch, span } => {
                let cond_op = self.lower_expr(condition);
                let then_bb = self.new_block();
                let else_bb = self.new_block();
                let merge_bb = self.new_block();
                let result_temp = self.new_temp();

                self.terminate(Terminator::Branch {
                    cond: cond_op,
                    then_block: then_bb,
                    else_block: else_bb,
                    span: *span,
                });

                // Then branch
                self.current_bb = then_bb;
                self.lower_block(then_branch);
                if let Some(t) = &then_branch.trailing_expr {
                    let op = self.lower_expr(t);
                    self.emit_stmt(HirStatement::Assign {
                        place: Place::Local(result_temp.clone()),
                        rvalue: Rvalue::Use(op),
                        span: *span,
                    });
                }
                if !self.is_current_terminated() {
                    self.terminate(Terminator::Jump(merge_bb, *span));
                }

                // Else branch
                self.current_bb = else_bb;
                if let Some(eb) = else_branch {
                    self.lower_block(eb);
                    if let Some(t) = &eb.trailing_expr {
                        let op = self.lower_expr(t);
                        self.emit_stmt(HirStatement::Assign {
                            place: Place::Local(result_temp.clone()),
                            rvalue: Rvalue::Use(op),
                            span: *span,
                        });
                    }
                }
                if !self.is_current_terminated() {
                    self.terminate(Terminator::Jump(merge_bb, *span));
                }

                self.current_bb = merge_bb;
                Operand::Copy(Place::Local(result_temp))
            }
            Expr::Array(elements, span) => {
                let mut ops = Vec::new();
                for e in elements {
                    ops.push(self.lower_expr(e));
                }
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Array(ops),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::Index { target, index, span } => {
                let t_op = self.lower_expr(target);
                let i_op = self.lower_expr(index);
                let target_place = match t_op {
                    Operand::Copy(p) => p,
                    Operand::Constant(_) => {
                        let temp = self.new_temp();
                        self.emit_stmt(HirStatement::Assign {
                            place: Place::Local(temp.clone()),
                            rvalue: Rvalue::Use(t_op),
                            span: *span,
                        });
                        Place::Local(temp)
                    }
                };
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Use(Operand::Copy(Place::Index {
                        base: Box::new(target_place),
                        index: Box::new(i_op),
                    })),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::Slice { target, start, end, span } => {
                let t_op = self.lower_expr(target);
                if let Some(s) = start {
                    let _ = self.lower_expr(s);
                }
                if let Some(e) = end {
                    let _ = self.lower_expr(e);
                }
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Use(t_op),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::IndexAssign { target, index, value, span } => {
                let t_op = self.lower_expr(target);
                let i_op = self.lower_expr(index);
                let v_op = self.lower_expr(value);
                let target_place = match t_op {
                    Operand::Copy(p) => p,
                    Operand::Constant(_) => Place::Local(self.new_temp()),
                };
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Index {
                        base: Box::new(target_place),
                        index: Box::new(i_op),
                    },
                    rvalue: Rvalue::Use(v_op.clone()),
                    span: *span,
                });
                v_op
            }
            Expr::StructInit { name, fields, span, .. } => {
                let mut field_ops = Vec::new();
                for (fname, fexpr) in fields {
                    field_ops.push((fname.clone(), self.lower_expr(fexpr)));
                }
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Struct(name.clone(), field_ops),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::FieldAccess { target, field, span } => {
                let t_op = self.lower_expr(target);
                let target_place = match t_op {
                    Operand::Copy(p) => p,
                    Operand::Constant(_) => Place::Local(self.new_temp()),
                };
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Use(Operand::Copy(Place::Field {
                        base: Box::new(target_place),
                        field: field.clone(),
                    })),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::FieldAssign { target, field, value, span } => {
                let t_op = self.lower_expr(target);
                let v_op = self.lower_expr(value);
                let target_place = match t_op {
                    Operand::Copy(p) => p,
                    Operand::Constant(_) => Place::Local(self.new_temp()),
                };
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Field {
                        base: Box::new(target_place),
                        field: field.clone(),
                    },
                    rvalue: Rvalue::Use(v_op.clone()),
                    span: *span,
                });
                v_op
            }
            Expr::MethodCall { target, method, args, span, .. } => {
                let t_op = self.lower_expr(target);
                let mut lowered_args = vec![t_op];
                for a in args {
                    lowered_args.push(self.lower_expr(a));
                }
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Call {
                    dest: Some(Place::Local(temp.clone())),
                    callee: method.clone(),
                    args: lowered_args,
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::Path { base, member, .. } => {
                Operand::Copy(Place::Local(format!("{}_{}", base, member)))
            }
            Expr::Match { subject, arms, span } => {
                let subj_op = self.lower_expr(subject);
                let result_temp = self.new_temp();
                let merge_bb = self.new_block();

                for arm in arms {
                    let arm_body_bb = self.new_block();
                    let next_arm_bb = self.new_block();

                    let cond_temp = self.new_temp();
                    self.emit_stmt(HirStatement::Assign {
                        place: Place::Local(cond_temp.clone()),
                        rvalue: Rvalue::BinaryOp(
                            BinOp::Eq,
                            subj_op.clone(),
                            Operand::Constant(Value::Void), // simplified pattern test
                        ),
                        span: arm.span,
                    });

                    self.terminate(Terminator::Branch {
                        cond: Operand::Copy(Place::Local(cond_temp)),
                        then_block: arm_body_bb,
                        else_block: next_arm_bb,
                        span: arm.span,
                    });

                    self.current_bb = arm_body_bb;
                    let body_op = self.lower_expr(&arm.body);
                    self.emit_stmt(HirStatement::Assign {
                        place: Place::Local(result_temp.clone()),
                        rvalue: Rvalue::Use(body_op),
                        span: arm.span,
                    });
                    if !self.is_current_terminated() {
                        self.terminate(Terminator::Jump(merge_bb, arm.span));
                    }

                    self.current_bb = next_arm_bb;
                }

                if !self.is_current_terminated() {
                    self.terminate(Terminator::Jump(merge_bb, *span));
                }

                self.current_bb = merge_bb;
                Operand::Copy(Place::Local(result_temp))
            }
            Expr::Block(b) => {
                self.lower_block(b);
                if let Some(t) = &b.trailing_expr {
                    self.lower_expr(t)
                } else {
                    Operand::Constant(Value::Void)
                }
            }
            Expr::UnsafeBlock(b, _) => {
                self.lower_block(b);
                if let Some(t) = &b.trailing_expr {
                    self.lower_expr(t)
                } else {
                    Operand::Constant(Value::Void)
                }
            }
            Expr::Borrow { expr, is_mut, span } => {
                let place = match &**expr {
                    Expr::Ident(name, _) => Place::Local(name.clone()),
                    _ => {
                        let op = self.lower_expr(expr);
                        let temp = self.new_temp();
                        self.emit_stmt(HirStatement::Assign {
                            place: Place::Local(temp.clone()),
                            rvalue: Rvalue::Use(op),
                            span: *span,
                        });
                        Place::Local(temp)
                    }
                };
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Ref {
                        is_mut: *is_mut,
                        place,
                    },
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::Deref { expr, span } => {
                let op = self.lower_expr(expr);
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Use(op),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::DerefAssign { target, value, .. } => {
                let _tgt = self.lower_expr(target);
                self.lower_expr(value)
            }
            Expr::Try(expr, span) => {
                let op = self.lower_expr(expr);
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Use(op),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::Closure { span, .. } => {
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Use(Operand::Constant(Value::Void)),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::Tuple(elements, span) => {
                let operands: Vec<Operand> = elements.iter().map(|e| self.lower_expr(e)).collect();
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Array(operands),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::TupleIndex { target, index, span } => {
                let t_op = self.lower_expr(target);
                let target_place = match t_op {
                    Operand::Copy(p) => p,
                    Operand::Constant(_) => Place::Local(self.new_temp()),
                };
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Use(Operand::Copy(Place::Field {
                        base: Box::new(target_place),
                        field: format!("_{}", index),
                    })),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
            Expr::TupleIndexAssign { target, index, value, span } => {
                let t_op = self.lower_expr(target);
                let v_op = self.lower_expr(value);
                let target_place = match t_op {
                    Operand::Copy(p) => p,
                    Operand::Constant(_) => Place::Local(self.new_temp()),
                };
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Field {
                        base: Box::new(target_place),
                        field: format!("_{}", index),
                    },
                    rvalue: Rvalue::Use(v_op),
                    span: *span,
                });
                Operand::Copy(Place::Local(self.new_temp()))
            }
            Expr::FormatString(parts, span) => {
                for part in parts {
                    if let FormatPart::Expr(e) = part {
                        let _ = self.lower_expr(e);
                    }
                }
                let temp = self.new_temp();
                self.emit_stmt(HirStatement::Assign {
                    place: Place::Local(temp.clone()),
                    rvalue: Rvalue::Use(Operand::Constant(Value::Str(String::new()))),
                    span: *span,
                });
                Operand::Copy(Place::Local(temp))
            }
        }
    }
}
