use std::collections::{HashMap, HashSet};
use crate::ast::{BinOp, Block, Expr, FormatPart, Item, Program, Stmt, UnOp};
use crate::eval::Value;
use crate::hir::{BasicBlockId, HirFunction, HirProgram, HirStatement, Operand, Place, Rvalue, Terminator};

/// Folds a binary operation on two known constant values.
pub fn fold_binop(op: BinOp, left: &Value, right: &Value) -> Option<Value> {
    match (left, right) {
        (Value::Int(l), Value::Int(r)) => match op {
            BinOp::Add => Some(Value::Int(l.wrapping_add(*r))),
            BinOp::Sub => Some(Value::Int(l.wrapping_sub(*r))),
            BinOp::Mul => Some(Value::Int(l.wrapping_mul(*r))),
            BinOp::Div if *r != 0 => Some(Value::Int(l.wrapping_div(*r))),
            BinOp::Mod if *r != 0 => Some(Value::Int(l.wrapping_rem(*r))),
            BinOp::Eq => Some(Value::Bool(l == r)),
            BinOp::NotEq => Some(Value::Bool(l != r)),
            BinOp::Lt => Some(Value::Bool(l < r)),
            BinOp::LtEq => Some(Value::Bool(l <= r)),
            BinOp::Gt => Some(Value::Bool(l > r)),
            BinOp::GtEq => Some(Value::Bool(l >= r)),
            _ => None,
        },
        (Value::Float(l), Value::Float(r)) => match op {
            BinOp::Add => Some(Value::Float(l + r)),
            BinOp::Sub => Some(Value::Float(l - r)),
            BinOp::Mul => Some(Value::Float(l * r)),
            BinOp::Div if *r != 0.0 => Some(Value::Float(l / r)),
            BinOp::Eq => Some(Value::Bool((l - r).abs() < f64::EPSILON)),
            BinOp::NotEq => Some(Value::Bool((l - r).abs() >= f64::EPSILON)),
            BinOp::Lt => Some(Value::Bool(l < r)),
            BinOp::LtEq => Some(Value::Bool(l <= r)),
            BinOp::Gt => Some(Value::Bool(l > r)),
            BinOp::GtEq => Some(Value::Bool(l >= r)),
            _ => None,
        },
        (Value::Bool(l), Value::Bool(r)) => match op {
            BinOp::And => Some(Value::Bool(*l && *r)),
            BinOp::Or => Some(Value::Bool(*l || *r)),
            BinOp::Eq => Some(Value::Bool(l == r)),
            BinOp::NotEq => Some(Value::Bool(l != r)),
            _ => None,
        },
        (Value::Char(l), Value::Char(r)) => match op {
            BinOp::Eq => Some(Value::Bool(l == r)),
            BinOp::NotEq => Some(Value::Bool(l != r)),
            BinOp::Lt => Some(Value::Bool(l < r)),
            BinOp::LtEq => Some(Value::Bool(l <= r)),
            BinOp::Gt => Some(Value::Bool(l > r)),
            BinOp::GtEq => Some(Value::Bool(l >= r)),
            _ => None,
        },
        (Value::U8(l), Value::U8(r)) => match op {
            BinOp::Add => Some(Value::U8(l.wrapping_add(*r))),
            BinOp::Sub => Some(Value::U8(l.wrapping_sub(*r))),
            BinOp::Mul => Some(Value::U8(l.wrapping_mul(*r))),
            BinOp::Div if *r != 0 => Some(Value::U8(l.wrapping_div(*r))),
            BinOp::Mod if *r != 0 => Some(Value::U8(l.wrapping_rem(*r))),
            BinOp::Eq => Some(Value::Bool(l == r)),
            BinOp::NotEq => Some(Value::Bool(l != r)),
            BinOp::Lt => Some(Value::Bool(l < r)),
            BinOp::LtEq => Some(Value::Bool(l <= r)),
            BinOp::Gt => Some(Value::Bool(l > r)),
            BinOp::GtEq => Some(Value::Bool(l >= r)),
            _ => None,
        },
        (Value::Str(l), Value::Str(r)) => match op {
            BinOp::Add => Some(Value::Str(format!("{}{}", l, r))),
            BinOp::Eq => Some(Value::Bool(l == r)),
            BinOp::NotEq => Some(Value::Bool(l != r)),
            _ => None,
        },
        _ => None,
    }
}

/// Folds a unary operation on a known constant value.
pub fn fold_unop(op: UnOp, val: &Value) -> Option<Value> {
    match (op, val) {
        (UnOp::Neg, Value::Int(i)) => Some(Value::Int(-i)),
        (UnOp::Neg, Value::Float(f)) => Some(Value::Float(-f)),
        (UnOp::Not, Value::Bool(b)) => Some(Value::Bool(!b)),
        _ => None,
    }
}

/// Optimizes an AST expression recursively via Constant Folding.
pub fn optimize_ast_expr(expr: &mut Expr) {
    match expr {
        Expr::Binary { op, left, right, span } => {
            optimize_ast_expr(left);
            optimize_ast_expr(right);

            let l_const = extract_const_from_ast(left);
            let r_const = extract_const_from_ast(right);

            if let (Some(l_val), Some(r_val)) = (l_const, r_const) {
                if let Some(folded) = fold_binop(*op, &l_val, &r_val) {
                    *expr = value_to_ast_expr(folded, *span);
                }
            }
        }
        Expr::Unary { op, expr: inner, span } => {
            optimize_ast_expr(inner);
            if let Some(inner_val) = extract_const_from_ast(inner) {
                if let Some(folded) = fold_unop(*op, &inner_val) {
                    *expr = value_to_ast_expr(folded, *span);
                }
            }
        }
        Expr::If { condition, then_branch, else_branch, span: _ } => {
            optimize_ast_expr(condition);
            optimize_ast_block(then_branch);
            if let Some(el) = else_branch {
                optimize_ast_block(el);
            }

            // If condition folded into a constant boolean, simplify!
            if let Some(Value::Bool(b)) = extract_const_from_ast(condition) {
                if b {
                    *expr = Expr::Block(then_branch.clone());
                } else if let Some(el) = else_branch {
                    *expr = Expr::Block(el.clone());
                }
            }
        }
        Expr::Block(block) => {
            optimize_ast_block(block);
        }
        Expr::Array(elements, _) => {
            for e in elements {
                optimize_ast_expr(e);
            }
        }
        Expr::Index { target, index, .. } => {
            optimize_ast_expr(target);
            optimize_ast_expr(index);
        }
        Expr::Slice { target, start, end, .. } => {
            optimize_ast_expr(target);
            if let Some(s) = start {
                optimize_ast_expr(s);
            }
            if let Some(e) = end {
                optimize_ast_expr(e);
            }
        }
        Expr::IndexAssign { target, index, value, .. } => {
            optimize_ast_expr(target);
            optimize_ast_expr(index);
            optimize_ast_expr(value);
        }
        Expr::FieldAssign { target, value, .. } => {
            optimize_ast_expr(target);
            optimize_ast_expr(value);
        }
        Expr::Assign { value, .. } => {
            optimize_ast_expr(value);
        }
        Expr::MethodCall { target, args, .. } => {
            optimize_ast_expr(target);
            for a in args {
                optimize_ast_expr(a);
            }
        }
        Expr::Call { args, .. } => {
            for a in args {
                optimize_ast_expr(a);
            }
        }
        Expr::StructInit { fields, .. } => {
            for (_, val) in fields {
                optimize_ast_expr(val);
            }
        }
        Expr::Borrow { expr: inner, .. } => {
            optimize_ast_expr(inner);
        }
        Expr::Deref { expr: inner, .. } => {
            optimize_ast_expr(inner);
        }
        Expr::Try(inner, ..) => {
            optimize_ast_expr(inner);
        }
        Expr::UnsafeBlock(block, ..) => {
            optimize_ast_block(block);
        }
        Expr::Closure { body, .. } => {
            optimize_ast_block(body);
        }
        Expr::Match { subject, arms, .. } => {
            optimize_ast_expr(subject);
            for arm in arms {
                optimize_ast_expr(&mut arm.body);
            }
        }
        Expr::Tuple(elements, _) => {
            for elem in elements {
                optimize_ast_expr(elem);
            }
        }
        Expr::TupleIndex { target, .. } => {
            optimize_ast_expr(target);
        }
        Expr::TupleIndexAssign { target, value, .. } => {
            optimize_ast_expr(target);
            optimize_ast_expr(value);
        }
        Expr::FormatString(parts, _) => {
            for part in parts {
                if let FormatPart::Expr(e) = part {
                    optimize_ast_expr(e);
                }
            }
        }
        _ => {}
    }
}

pub fn optimize_ast_block(block: &mut Block) {
    for stmt in &mut block.stmts {
        match stmt {
            Stmt::Let { init, .. } | Stmt::LetTuple { init, .. } => {
                if let Some(i) = init {
                    optimize_ast_expr(i);
                }
            }
            Stmt::Expr(e) => {
                optimize_ast_expr(e);
            }
            Stmt::While { condition, body, .. } => {
                optimize_ast_expr(condition);
                optimize_ast_block(body);
            }
            Stmt::ForRange { start, end, body, .. } => {
                optimize_ast_expr(start);
                optimize_ast_expr(end);
                optimize_ast_block(body);
            }
            Stmt::ForIn { iterable, body, .. } => {
                optimize_ast_expr(iterable);
                optimize_ast_block(body);
            }
            Stmt::Return(Some(e), ..) => {
                optimize_ast_expr(e);
            }
            Stmt::Const(c) => {
                optimize_ast_expr(&mut c.value);
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None, _) => {}
        }
    }
    if let Some(ref mut trailing) = block.trailing_expr {
        optimize_ast_expr(trailing);
    }
}

pub fn optimize_ast_program(program: &mut Program) {
    for item in &mut program.items {
        match item {
            Item::Function(f) => {
                optimize_ast_block(&mut f.body);
            }
            Item::Impl(imp) => {
                for m in &mut imp.methods {
                    optimize_ast_block(&mut m.body);
                }
            }
            Item::Const(c) => {
                optimize_ast_expr(&mut c.value);
            }
            _ => {}
        }
    }
}

fn extract_const_from_ast(expr: &Expr) -> Option<Value> {
    match expr {
        Expr::Int(i, _) => Some(Value::Int(*i)),
        Expr::Float(f, _) => Some(Value::Float(*f)),
        Expr::Bool(b, _) => Some(Value::Bool(*b)),
        Expr::Str(s, _) => Some(Value::Str(s.clone())),
        Expr::Char(c, _) => Some(Value::Char(*c)),
        Expr::U8(b, _) => Some(Value::U8(*b)),
        _ => None,
    }
}

fn value_to_ast_expr(val: Value, span: crate::span::Span) -> Expr {
    match val {
        Value::Int(i) => Expr::Int(i, span),
        Value::Float(f) => Expr::Float(f, span),
        Value::Bool(b) => Expr::Bool(b, span),
        Value::Str(s) => Expr::Str(s, span),
        Value::Char(c) => Expr::Char(c, span),
        Value::U8(b) => Expr::U8(b, span),
        _ => Expr::Int(0, span),
    }
}

// ==========================================
// HIR OPTIMIZATIONS (CFG & Dataflow)
// ==========================================

/// Optimizes an entire HIR program.
pub fn optimize_hir_program(prog: &mut HirProgram) {
    for func in &mut prog.functions {
        optimize_hir_function(func);
    }
}

/// Optimizes a single HIR function through multiple passes:
/// 1. Local constant propagation & binary/unary folding.
/// 2. Constant branch simplification (`branch true/false` -> `jump`).
/// 3. Dead basic block elimination (unreachable CFG pruning).
pub fn optimize_hir_function(func: &mut HirFunction) {
    let mut changed = true;
    let mut iteration = 0;

    while changed && iteration < 10 {
        changed = false;
        iteration += 1;

        // Pass 1: Dead Basic Block Elimination first
        if eliminate_dead_blocks(func) {
            changed = true;
        }

        // Pass 2: Constant folding & propagation within basic blocks
        for block in &mut func.blocks {
            let mut block_constants: HashMap<String, Value> = HashMap::new();

            for stmt in &mut block.statements {
                if let HirStatement::Assign { place, rvalue, .. } = stmt {
                    // Constant folding on rvalue
                    fold_rvalue(rvalue, &block_constants);

                    if let Place::Local(var_name) = place {
                        if let Rvalue::Use(Operand::Constant(c)) = rvalue {
                            block_constants.insert(var_name.clone(), c.clone());
                        } else {
                            block_constants.remove(var_name);
                        }
                    }
                }
            }

            // Pass 3: Branch simplification
            match &mut block.terminator {
                Terminator::Branch { cond, then_block, else_block, span } => {
                    let cond_val = match cond {
                        Operand::Constant(Value::Bool(b)) => Some(*b),
                        Operand::Copy(Place::Local(name)) => {
                            if let Some(Value::Bool(b)) = block_constants.get(name) {
                                Some(*b)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    if let Some(b) = cond_val {
                        let target = if b { *then_block } else { *else_block };
                        block.terminator = Terminator::Jump(target, *span);
                        changed = true;
                    }
                }
                _ => {}
            }
        }
    }
}

fn fold_rvalue(rvalue: &mut Rvalue, known_constants: &HashMap<String, Value>) {
    match rvalue {
        Rvalue::Use(op) => {
            resolve_operand(op, known_constants);
        }
        Rvalue::UnaryOp(op, operand) => {
            resolve_operand(operand, known_constants);
            if let Operand::Constant(c) = operand {
                if let Some(folded) = fold_unop(*op, c) {
                    *rvalue = Rvalue::Use(Operand::Constant(folded));
                }
            }
        }
        Rvalue::BinaryOp(op, left, right) => {
            resolve_operand(left, known_constants);
            resolve_operand(right, known_constants);

            if let (Operand::Constant(l_c), Operand::Constant(r_c)) = (&left, &right) {
                if let Some(folded) = fold_binop(*op, l_c, r_c) {
                    *rvalue = Rvalue::Use(Operand::Constant(folded));
                }
            }
        }
        _ => {}
    }
}

fn resolve_operand(op: &mut Operand, known_constants: &HashMap<String, Value>) {
    if let Operand::Copy(Place::Local(name)) = op {
        if let Some(val) = known_constants.get(name) {
            *op = Operand::Constant(val.clone());
        }
    }
}

/// Discovers reachable blocks from `start_block` and removes unreachable ones.
fn eliminate_dead_blocks(func: &mut HirFunction) -> bool {
    let mut reachable: HashSet<BasicBlockId> = HashSet::new();
    let mut worklist: Vec<BasicBlockId> = vec![func.start_block];
    reachable.insert(func.start_block);

    let block_map: HashMap<BasicBlockId, &crate::hir::BasicBlock> =
        func.blocks.iter().map(|b| (b.id, b)).collect();

    while let Some(current_id) = worklist.pop() {
        if let Some(block) = block_map.get(&current_id) {
            let successors = match &block.terminator {
                Terminator::Jump(target, _) => vec![*target],
                Terminator::Branch { then_block, else_block, .. } => vec![*then_block, *else_block],
                Terminator::Return(..) | Terminator::Unreachable => vec![],
            };

            for succ in successors {
                if reachable.insert(succ) {
                    worklist.push(succ);
                }
            }
        }
    }

    let orig_count = func.blocks.len();
    func.blocks.retain(|b| reachable.contains(&b.id));
    func.blocks.len() < orig_count
}
