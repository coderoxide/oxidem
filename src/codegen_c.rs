use crate::ast::*;
use std::collections::HashMap;

fn escape_c_string(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn escape_c_char(c: char) -> String {
    match c {
        '\'' => "\\'".to_string(),
        '\\' => "\\\\".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        '\0' => "\\0".to_string(),
        other => other.to_string(),
    }
}

fn is_libc_builtin(name: &str) -> bool {
    matches!(
        name,
        "sqrt" | "pow" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "atan2"
            | "floor" | "ceil" | "round" | "log" | "exp" | "fabs" | "abs"
            | "strlen" | "strcmp" | "puts" | "printf" | "free" | "exit"
    )
}

fn expr_has_str(e: &Expr) -> bool {
    match e {
        Expr::Str(..) | Expr::FormatString(..) => true,
        Expr::Binary { left, right, .. } => expr_has_str(left) || expr_has_str(right),
        _ => false,
    }
}

pub struct CodegenC {
    output: String,
    indent: usize,
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, EnumDef>,
    var_types: HashMap<String, String>,
    var_ox_types: HashMap<String, Type>,
    method_to_struct: HashMap<String, String>,
    current_impl: Option<String>,
    current_fn_ret: Option<Type>,
    tuples: Vec<Vec<Type>>,
    temp_counter: usize,
    fn_ret_types: HashMap<String, Type>,
    fn_param_types: HashMap<String, Vec<Type>>,
}

impl CodegenC {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            structs: HashMap::new(),
            enums: HashMap::new(),
            var_types: HashMap::new(),
            var_ox_types: HashMap::new(),
            method_to_struct: HashMap::new(),
            current_impl: None,
            current_fn_ret: None,
            tuples: Vec::new(),
            temp_counter: 0,
            fn_ret_types: HashMap::new(),
            fn_param_types: HashMap::new(),
        }
    }

    fn write_line(&mut self, text: &str) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        self.output.push_str(text);
        self.output.push('\n');
    }

    fn c_tuple_name(&self, types: &[Type]) -> String {
        let mangled = types.iter().map(crate::monomorph::mangle_type).collect::<Vec<_>>().join("_");
        format!("_OxTuple_{}", mangled)
    }

    fn infer_expr_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Int(..) => Type::Int,
            Expr::Float(..) => Type::Float,
            Expr::Bool(..) => Type::Bool,
            Expr::Str(..) => Type::Str,
            Expr::Char(..) => Type::Char,
            Expr::U8(..) => Type::U8,
            Expr::Ident(name, _) => {
                if name == "self" {
                    if let Some(ref target) = self.current_impl {
                        return Type::Named(target.clone());
                    }
                }
                self.var_ox_types.get(name).cloned().unwrap_or(Type::Int)
            }
            Expr::Array(elements, _) => {
                if let Some(first) = elements.first() {
                    Type::Array(Box::new(self.infer_expr_type(first)))
                } else {
                    Type::Array(Box::new(Type::Int))
                }
            }
            Expr::Tuple(elements, _) => {
                Type::Tuple(elements.iter().map(|e| self.infer_expr_type(e)).collect())
            }
            Expr::TupleIndex { target, index, .. } => {
                match self.infer_expr_type(target) {
                    Type::Tuple(elem_tys) => elem_tys.get(*index).cloned().unwrap_or(Type::Int),
                    _ => Type::Int,
                }
            }
            Expr::Call { callee, args, .. } => {
                match callee.as_str() {
                    "Box::new" => {
                        if let Some(arg) = args.first() {
                            Type::Box(Box::new(self.infer_expr_type(arg)))
                        } else {
                            Type::Void
                        }
                    }
                    "Rc::new" => {
                        if let Some(arg) = args.first() {
                            Type::Rc(Box::new(self.infer_expr_type(arg)))
                        } else {
                            Type::Void
                        }
                    }
                    "Rc::strong_count" => Type::Int,
                    "RefCell::new" => {
                        if let Some(arg) = args.first() {
                            Type::RefCell(Box::new(self.infer_expr_type(arg)))
                        } else {
                            Type::Void
                        }
                    }
                    "to_int" | "len" => Type::Int,
                    "to_float" => Type::Float,
                    "to_char" => Type::Char,
                    "to_u8" | "to_byte" => Type::U8,
                    _ => self.fn_ret_types.get(callee).cloned().unwrap_or(Type::Void),
                }
            }
            Expr::Deref { expr, .. } => {
                let inner_t = self.infer_expr_type(expr);
                match inner_t {
                    Type::Box(t) | Type::Rc(t) | Type::RefCell(t) => *t,
                    Type::Ref { inner, .. } | Type::RawPtr { inner, .. } => *inner,
                    other => other,
                }
            }
            Expr::DerefAssign { .. } => Type::Void,
            Expr::MethodCall { target, method, .. } => {
                let target_ty = self.infer_expr_type(target);
                if method == "unwrap" {
                    if let Type::Option(inner) = target_ty {
                        return *inner;
                    }
                    if let Type::Result(ok, _) = target_ty {
                        return *ok;
                    }
                }
                if method == "clone" {
                    if let Type::Rc(inner) = target_ty {
                        return Type::Rc(inner);
                    }
                }
                let mut curr_target = target_ty;
                while let Type::Box(in_t) | Type::Rc(in_t) | Type::RefCell(in_t) = curr_target {
                    curr_target = *in_t;
                }
                let struct_name = match curr_target {
                    Type::Named(s) => s,
                    _ => self.method_to_struct.get(method).cloned().unwrap_or_default(),
                };
                let key = format!("{}_{}", struct_name, method);
                self.fn_ret_types.get(&key).or_else(|| self.fn_ret_types.get(method)).cloned().unwrap_or(Type::Void)
            }
            Expr::FieldAccess { target, field, .. } => {
                let mut target_ty = self.infer_expr_type(target);
                while let Type::Box(in_t) | Type::Rc(in_t) | Type::RefCell(in_t) = target_ty {
                    target_ty = *in_t;
                }
                if let Type::Named(s_name) = target_ty {
                    if let Some(s) = self.structs.get(&s_name) {
                        if let Some((_, fty)) = s.fields.iter().find(|(k, _)| k == field) {
                            return fty.clone();
                        }
                    }
                }
                Type::Int
            }
            Expr::Binary { op, left, .. } => {
                let left_ty = self.infer_expr_type(left);
                if let Type::Named(ref s_name) = left_ty {
                    let method = match op {
                        BinOp::Add => Some("add"),
                        BinOp::Sub => Some("sub"),
                        BinOp::Mul => Some("mul"),
                        BinOp::Div => Some("div"),
                        BinOp::Mod => Some("rem"),
                        BinOp::Eq => Some("eq"),
                        BinOp::NotEq => Some("ne"),
                        BinOp::Lt => Some("lt"),
                        BinOp::LtEq => Some("le"),
                        BinOp::Gt => Some("gt"),
                        BinOp::GtEq => Some("ge"),
                        _ => None,
                    };
                    if let Some(m) = method {
                        let key = format!("{}_{}", s_name, m);
                        if let Some(ret) = self.fn_ret_types.get(&key) {
                            return ret.clone();
                        } else if *op == BinOp::NotEq || *op == BinOp::Eq {
                            return Type::Bool;
                        }
                    }
                }
                match op {
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq | BinOp::And | BinOp::Or => Type::Bool,
                    _ => left_ty,
                }
            }
            Expr::Unary { op, expr, .. } => {
                let inner_ty = self.infer_expr_type(expr);
                if let Type::Named(ref s_name) = inner_ty {
                    let m_name = match op {
                        UnOp::Neg => "neg",
                        UnOp::Not => "not",
                    };
                    let key = format!("{}_{}", s_name, m_name);
                    if let Some(ret) = self.fn_ret_types.get(&key) {
                        return ret.clone();
                    }
                }
                match op {
                    UnOp::Not => Type::Bool,
                    UnOp::Neg => inner_ty,
                }
            }
            Expr::FormatString(..) => Type::Str,
            Expr::Slice { target, .. } => {
                match self.infer_expr_type(target) {
                    Type::Array(inner) | Type::Slice(inner) => Type::Slice(inner),
                    Type::Str | Type::StrSlice => Type::StrSlice,
                    other => Type::Slice(Box::new(other)),
                }
            }
            Expr::Index { target, .. } => {
                match self.infer_expr_type(target) {
                    Type::Array(inner) | Type::Slice(inner) => *inner,
                    Type::Str | Type::StrSlice => Type::Char,
                    Type::Named(ref s_name) => {
                        let key = format!("{}_index", s_name);
                        self.fn_ret_types.get(&key).cloned().unwrap_or(Type::Int)
                    }
                    _ => Type::Int,
                }
            }
            Expr::StructInit { name, .. } => Type::Named(name.clone()),
            Expr::Block(b) | Expr::UnsafeBlock(b, _) => {
                if let Some(t) = &b.trailing_expr {
                    self.infer_expr_type(t)
                } else {
                    Type::Void
                }
            }
            Expr::If { then_branch, .. } => {
                if let Some(t) = &then_branch.trailing_expr {
                    self.infer_expr_type(t)
                } else {
                    Type::Void
                }
            }
            _ => Type::Void,
        }
    }

    fn collect_tuples_from_type(&mut self, ty: &Type) {
        match ty {
            Type::Tuple(elem_tys) => {
                for t in elem_tys {
                    self.collect_tuples_from_type(t);
                }
                if !self.tuples.iter().any(|existing| existing == elem_tys) {
                    self.tuples.push(elem_tys.clone());
                }
            }
            Type::Array(inner) | Type::Slice(inner) | Type::Option(inner) | Type::Ref { inner, .. } | Type::RawPtr { inner, .. } => {
                self.collect_tuples_from_type(inner);
            }
            Type::Result(ok, err) => {
                self.collect_tuples_from_type(ok);
                self.collect_tuples_from_type(err);
            }
            Type::Fn { params, ret } => {
                for p in params {
                    self.collect_tuples_from_type(p);
                }
                self.collect_tuples_from_type(ret);
            }
            _ => {}
        }
    }

    fn collect_tuples_from_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Tuple(elements, _) => {
                for e in elements {
                    self.collect_tuples_from_expr(e);
                }
                let ty = self.infer_expr_type(expr);
                self.collect_tuples_from_type(&ty);
            }
            Expr::TupleIndex { target, .. } => {
                self.collect_tuples_from_expr(target);
            }
            Expr::Slice { target, start, end, .. } => {
                self.collect_tuples_from_expr(target);
                if let Some(s) = start { self.collect_tuples_from_expr(s); }
                if let Some(e) = end { self.collect_tuples_from_expr(e); }
            }
            Expr::TupleIndexAssign { target, value, .. } => {
                self.collect_tuples_from_expr(target);
                self.collect_tuples_from_expr(value);
            }
            Expr::Binary { left, right, .. } => {
                self.collect_tuples_from_expr(left);
                self.collect_tuples_from_expr(right);
            }
            Expr::Unary { expr, .. } | Expr::Borrow { expr, .. } | Expr::Deref { expr, .. } | Expr::Try(expr, ..) => {
                self.collect_tuples_from_expr(expr);
            }
            Expr::DerefAssign { target, value, .. } => {
                self.collect_tuples_from_expr(target);
                self.collect_tuples_from_expr(value);
            }
            Expr::Call { args, .. } => {
                for a in args {
                    self.collect_tuples_from_expr(a);
                }
            }
            Expr::MethodCall { target, args, .. } => {
                self.collect_tuples_from_expr(target);
                for a in args {
                    self.collect_tuples_from_expr(a);
                }
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.collect_tuples_from_expr(condition);
                self.collect_tuples_from_block(then_branch);
                if let Some(eb) = else_branch {
                    self.collect_tuples_from_block(eb);
                }
            }
            Expr::Block(b) | Expr::UnsafeBlock(b, _) => {
                self.collect_tuples_from_block(b);
            }
            Expr::Match { subject, arms, .. } => {
                self.collect_tuples_from_expr(subject);
                for arm in arms {
                    self.collect_tuples_from_expr(&arm.body);
                }
            }
            Expr::Array(elements, _) => {
                for e in elements {
                    self.collect_tuples_from_expr(e);
                }
            }
            Expr::StructInit { fields, .. } => {
                for (_, fval) in fields {
                    self.collect_tuples_from_expr(fval);
                }
            }
            Expr::FieldAssign { target, value, .. } => {
                self.collect_tuples_from_expr(target);
                self.collect_tuples_from_expr(value);
            }
            Expr::Assign { value, .. } => {
                self.collect_tuples_from_expr(value);
            }
            Expr::FormatString(parts, _) => {
                for part in parts {
                    if let FormatPart::Expr(e) = part {
                        self.collect_tuples_from_expr(e);
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_tuples_from_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { ty, init, .. } => {
                if let Some(t) = ty {
                    self.collect_tuples_from_type(t);
                }
                if let Some(e) = init {
                    self.collect_tuples_from_expr(e);
                }
            }
            Stmt::LetTuple { ty, init, .. } => {
                if let Some(t) = ty {
                    self.collect_tuples_from_type(t);
                }
                if let Some(e) = init {
                    self.collect_tuples_from_expr(e);
                }
            }
            Stmt::Expr(e) => {
                self.collect_tuples_from_expr(e);
            }
            Stmt::While { condition, body, .. } => {
                self.collect_tuples_from_expr(condition);
                self.collect_tuples_from_block(body);
            }
            Stmt::ForRange { start, end, body, .. } => {
                self.collect_tuples_from_expr(start);
                self.collect_tuples_from_expr(end);
                self.collect_tuples_from_block(body);
            }
            Stmt::ForIn { iterable, body, .. } => {
                self.collect_tuples_from_expr(iterable);
                self.collect_tuples_from_block(body);
            }
            Stmt::Return(Some(e), ..) => {
                self.collect_tuples_from_expr(e);
            }
            _ => {}
        }
    }

    fn collect_tuples_from_block(&mut self, block: &Block) {
        for s in &block.stmts {
            self.collect_tuples_from_stmt(s);
        }
        if let Some(t) = &block.trailing_expr {
            self.collect_tuples_from_expr(t);
        }
    }

    fn to_c_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "int64_t".to_string(),
            Type::Float => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Str => "const char*".to_string(),
            Type::Char => "char".to_string(),
            Type::U8 => "uint8_t".to_string(),
            Type::Void => "void".to_string(),
            Type::Array(_) => "int64_t*".to_string(),
            Type::Slice(_) => "_OxSlice".to_string(),
            Type::StrSlice => "_OxStrSlice".to_string(),
            Type::Tuple(types) => self.c_tuple_name(types),
            Type::Named(name) => {
                if name == "Map" {
                    "_OxMap*".to_string()
                } else {
                    name.clone()
                }
            }
            Type::Ref { inner, is_mut } => {
                if let Type::Array(_) = **inner {
                    return "_OxSlice".to_string();
                }
                if let Type::Str = **inner {
                    return "_OxStrSlice".to_string();
                }
                let inner_c = self.to_c_type(inner);
                if *is_mut {
                    format!("{}*", inner_c)
                } else {
                    format!("const {}*", inner_c)
                }
            }
            Type::RawPtr { inner, is_mut } => {
                let inner_c = self.to_c_type(inner);
                if *is_mut {
                    format!("{}*", inner_c)
                } else {
                    format!("const {}*", inner_c)
                }
            }
            Type::Fn { .. } => "void*".to_string(),
            Type::Option(_) => "_OxOption".to_string(),
            Type::Result(_, _) => "_OxResult".to_string(),
            Type::Box(inner) => {
                let inner_c = self.to_c_type(inner);
                format!("{}*", inner_c)
            }
            Type::Rc(_) => "_OxRc*".to_string(),
            Type::RefCell(inner) => {
                let inner_c = self.to_c_type(inner);
                format!("{}*", inner_c)
            }
            Type::Generic { name, type_args } => {
                let args_str = type_args.iter().map(|a| match a {
                    Type::Int => "int".to_string(),
                    Type::Float => "float".to_string(),
                    Type::Bool => "bool".to_string(),
                    Type::Str => "str".to_string(),
                    Type::Char => "char".to_string(),
                    Type::U8 => "u8".to_string(),
                    Type::Named(n) => n.clone(),
                    _ => "val".to_string(),
                }).collect::<Vec<_>>().join("_");
                format!("{}__{}", name, args_str)
            }
        }
    }

    pub fn compile_program(mut self, program: &Program) -> String {
        // Collect structs, enums, functions and tuples first
        for item in &program.items {
            match item {
                Item::Struct(s) => {
                    self.structs.insert(s.name.clone(), s.clone());
                    for (_, fty) in &s.fields {
                        self.collect_tuples_from_type(fty);
                    }
                }
                Item::Enum(e) => {
                    self.enums.insert(e.name.clone(), e.clone());
                }
                Item::Function(f) => {
                    self.fn_ret_types.insert(f.name.clone(), f.return_type.clone());
                    self.fn_param_types.insert(f.name.clone(), f.params.iter().map(|p| p.ty.clone()).collect());
                    self.collect_tuples_from_type(&f.return_type);
                    for p in &f.params {
                        self.collect_tuples_from_type(&p.ty);
                    }
                    self.collect_tuples_from_block(&f.body);
                }
                Item::Impl(imp) => {
                    for m in &imp.methods {
                        let qualified = format!("{}_{}", imp.target, m.name);
                        self.fn_ret_types.insert(qualified.clone(), m.return_type.clone());
                        self.fn_ret_types.insert(m.name.clone(), m.return_type.clone());
                        self.fn_param_types.insert(qualified, m.params.iter().map(|p| p.ty.clone()).collect());
                        self.fn_param_types.insert(m.name.clone(), m.params.iter().map(|p| p.ty.clone()).collect());
                        self.method_to_struct.insert(m.name.clone(), imp.target.clone());
                        self.collect_tuples_from_type(&m.return_type);
                        for p in &m.params {
                            self.collect_tuples_from_type(&p.ty);
                        }
                        self.collect_tuples_from_block(&m.body);
                    }
                }
                _ => {}
            }
        }

        // Header includes
        self.write_line("/* Generated by Oxide Compiler */");
        self.write_line("#define _GNU_SOURCE");
        self.write_line("#include <stdio.h>");
        self.write_line("#include <stdlib.h>");
        self.write_line("#include <stdint.h>");
        self.write_line("#include <stdbool.h>");
        self.write_line("#include <string.h>");
        self.write_line("#include <assert.h>");
        self.write_line("#include <math.h>");
        self.write_line("");

        // Runtime helpers
        self.write_line("/* Oxide High-Performance Ring Buffer Memory Arena (Zero Leaks, Zero Malloc Overhead) */");
        self.write_line("#define _OX_ARENA_SIZE 1048576");
        self.write_line("static __thread char _ox_arena_buf[_OX_ARENA_SIZE];");
        self.write_line("static __thread size_t _ox_arena_pos = 0;");
        self.write_line("");
        self.write_line("static inline char* _ox_arena_alloc(size_t size) {");
        self.write_line("    size_t aligned = (size + 7) & ~7;");
        self.write_line("    if (_ox_arena_pos + aligned >= _OX_ARENA_SIZE) {");
        self.write_line("        _ox_arena_pos = 0; /* Ring buffer wrap-around */");
        self.write_line("    }");
        self.write_line("    char* ptr = &_ox_arena_buf[_ox_arena_pos];");
        self.write_line("    _ox_arena_pos += aligned;");
        self.write_line("    return ptr;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline char* _ox_str_concat(const char* a, const char* b) {");
        self.write_line("    size_t la = a ? strlen(a) : 0;");
        self.write_line("    size_t lb = b ? strlen(b) : 0;");
        self.write_line("    char* res = _ox_arena_alloc(la + lb + 1);");
        self.write_line("    if (a) memcpy(res, a, la);");
        self.write_line("    if (b) memcpy(res + la, b, lb);");
        self.write_line("    res[la + lb] = '\\0';");
        self.write_line("    return res;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline void _ox_print_int(int64_t x) { printf(\"%ld\", (long)x); }");
        self.write_line("static inline void _ox_print_double(double x) { printf(\"%g\", x); }");
        self.write_line("static inline void _ox_print_bool(bool x) { printf(\"%s\", x ? \"true\" : \"false\"); }");
        self.write_line("static inline void _ox_print_char(char x) { printf(\"%c\", x); }");
        self.write_line("static inline void _ox_print_u8(uint8_t x) { printf(\"%u\", (unsigned int)x); }");
        self.write_line("static inline void _ox_print_str(const char* x) { printf(\"%s\", x ? x : \"\"); }");
        self.write_line("static inline void _ox_print_ptr(void* x) { printf(\"%p\", x); }");
        self.write_line("");
        self.write_line("static inline char* _ox_to_str_int(int64_t x) {");
        self.write_line("    char* buf = _ox_arena_alloc(32);");
        self.write_line("    snprintf(buf, 32, \"%ld\", (long)x);");
        self.write_line("    return buf;");
        self.write_line("}");
        self.write_line("static inline char* _ox_to_str_double(double x) {");
        self.write_line("    char* buf = _ox_arena_alloc(32);");
        self.write_line("    snprintf(buf, 32, \"%g\", x);");
        self.write_line("    return buf;");
        self.write_line("}");
        self.write_line("static inline char* _ox_to_str_bool(bool x) { return (char*)(x ? \"true\" : \"false\"); }");
        self.write_line("static inline char* _ox_to_str_char(char x) {");
        self.write_line("    char* buf = _ox_arena_alloc(2);");
        self.write_line("    buf[0] = x;");
        self.write_line("    buf[1] = '\\0';");
        self.write_line("    return buf;");
        self.write_line("}");
        self.write_line("static inline char* _ox_to_str_u8(uint8_t x) {");
        self.write_line("    char* buf = _ox_arena_alloc(16);");
        self.write_line("    snprintf(buf, 16, \"%u\", (unsigned int)x);");
        self.write_line("    return buf;");
        self.write_line("}");
        self.write_line("static inline char* _ox_to_str_str(const char* x) { return (char*)(x ? x : \"\"); }");
        self.write_line("");
        self.write_line("typedef struct {");
        self.write_line("    int64_t len;");
        self.write_line("    int64_t cap;");
        self.write_line("} _OxArrayHeader;");
        self.write_line("");
        self.write_line("#define _OX_ARRAY_HDR(ptr) (((_OxArrayHeader*)(ptr)) - 1)");
        self.write_line("");
        self.write_line("static inline void* _ox_array_create_sized(int64_t count, size_t elem_size) {");
        self.write_line("    int64_t cap = count < 8 ? 8 : count * 2;");
        self.write_line("    size_t total = sizeof(_OxArrayHeader) + cap * elem_size;");
        self.write_line("    _OxArrayHeader* hdr = (_OxArrayHeader*)_ox_arena_alloc(total);");
        self.write_line("    hdr->len = count;");
        self.write_line("    hdr->cap = cap;");
        self.write_line("    return (void*)(hdr + 1);");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline int64_t* _ox_array_create(int64_t count) {");
        self.write_line("    return (int64_t*)_ox_array_create_sized(count, sizeof(int64_t));");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline int64_t _ox_arr_len(void* ptr) {");
        self.write_line("    if (!ptr) return 0;");
        self.write_line("    return _OX_ARRAY_HDR(ptr)->len;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline int64_t _ox_str_len(const char* s) {");
        self.write_line("    return s ? (int64_t)strlen(s) : 0LL;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline int64_t* _ox_arr_push_impl(int64_t* arr, int64_t val) {");
        self.write_line("    _OxArrayHeader* hdr = _OX_ARRAY_HDR(arr);");
        self.write_line("    if (hdr->len >= hdr->cap) {");
        self.write_line("        int64_t new_cap = hdr->cap * 2 + 4;");
        self.write_line("        _OxArrayHeader* new_hdr = (_OxArrayHeader*)_ox_arena_alloc(sizeof(_OxArrayHeader) + new_cap * sizeof(int64_t));");
        self.write_line("        new_hdr->len = hdr->len;");
        self.write_line("        new_hdr->cap = new_cap;");
        self.write_line("        int64_t* new_arr = (int64_t*)(new_hdr + 1);");
        self.write_line("        memcpy(new_arr, arr, hdr->len * sizeof(int64_t));");
        self.write_line("        new_arr[new_hdr->len++] = val;");
        self.write_line("        return new_arr;");
        self.write_line("    }");
        self.write_line("    arr[hdr->len++] = val;");
        self.write_line("    return arr;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline void _ox_print_array(int64_t* arr) {");
        self.write_line("    if (!arr) { printf(\"[]\"); return; }");
        self.write_line("    int64_t len = _OX_ARRAY_HDR(arr)->len;");
        self.write_line("    printf(\"[\");");
        self.write_line("    for (int64_t i = 0; i < len; i++) {");
        self.write_line("        if (i > 0) printf(\", \");");
        self.write_line("        printf(\"%ld\", (long)arr[i]);");
        self.write_line("    }");
        self.write_line("    printf(\"]\");");
        self.write_line("}");
        self.write_line("");
        self.write_line("typedef struct {");
        self.write_line("    int64_t* ptr;");
        self.write_line("    int64_t len;");
        self.write_line("} _OxSlice;");
        self.write_line("");
        self.write_line("typedef struct {");
        self.write_line("    const char* ptr;");
        self.write_line("    int64_t len;");
        self.write_line("} _OxStrSlice;");
        self.write_line("");
        self.write_line("static inline _OxSlice _ox_slice_create(int64_t* arr, int64_t start, int64_t end) {");
        self.write_line("    if (!arr) return (_OxSlice){ .ptr = NULL, .len = 0 };");
        self.write_line("    int64_t total = _OX_ARRAY_HDR(arr)->len;");
        self.write_line("    if (start < 0) start = 0;");
        self.write_line("    if (end > total) end = total;");
        self.write_line("    if (end < start) end = start;");
        self.write_line("    return (_OxSlice){ .ptr = arr + start, .len = end - start };");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline _OxSlice _ox_slice_sub(_OxSlice s, int64_t start, int64_t end) {");
        self.write_line("    if (!s.ptr) return (_OxSlice){ .ptr = NULL, .len = 0 };");
        self.write_line("    if (start < 0) start = 0;");
        self.write_line("    if (end > s.len) end = s.len;");
        self.write_line("    if (end < start) end = start;");
        self.write_line("    return (_OxSlice){ .ptr = s.ptr + start, .len = end - start };");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline _OxStrSlice _ox_str_slice_create(const char* s, int64_t start, int64_t end) {");
        self.write_line("    if (!s) return (_OxStrSlice){ .ptr = \"\", .len = 0 };");
        self.write_line("    int64_t total = (int64_t)strlen(s);");
        self.write_line("    if (start < 0) start = 0;");
        self.write_line("    if (end > total) end = total;");
        self.write_line("    if (end < start) end = start;");
        self.write_line("    return (_OxStrSlice){ .ptr = s + start, .len = end - start };");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline _OxStrSlice _ox_str_slice_sub(_OxStrSlice s, int64_t start, int64_t end) {");
        self.write_line("    if (!s.ptr) return (_OxStrSlice){ .ptr = \"\", .len = 0 };");
        self.write_line("    if (start < 0) start = 0;");
        self.write_line("    if (end > s.len) end = s.len;");
        self.write_line("    if (end < start) end = start;");
        self.write_line("    return (_OxStrSlice){ .ptr = s.ptr + start, .len = end - start };");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline int64_t _ox_slice_len(_OxSlice s) { return s.len; }");
        self.write_line("static inline int64_t _ox_str_slice_len(_OxStrSlice s) { return s.len; }");
        self.write_line("");
        self.write_line("static inline void _ox_print_slice(_OxSlice s) {");
        self.write_line("    printf(\"[\");");
        self.write_line("    for (int64_t i = 0; i < s.len; i++) {");
        self.write_line("        if (i > 0) printf(\", \");");
        self.write_line("        printf(\"%ld\", (long)s.ptr[i]);");
        self.write_line("    }");
        self.write_line("    printf(\"]\");");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline void _ox_print_str_slice(_OxStrSlice s) {");
        self.write_line("    if (s.ptr && s.len > 0) {");
        self.write_line("        fwrite(s.ptr, 1, s.len, stdout);");
        self.write_line("    }");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline char* _ox_to_str_slice(_OxSlice s) {");
        self.write_line("    char* buf = _ox_arena_alloc(s.len * 32 + 16);");
        self.write_line("    int pos = snprintf(buf, 16, \"[\");");
        self.write_line("    for (int64_t i = 0; i < s.len; i++) {");
        self.write_line("        if (i > 0) pos += snprintf(buf + pos, 4, \", \");");
        self.write_line("        pos += snprintf(buf + pos, 32, \"%ld\", (long)s.ptr[i]);");
        self.write_line("    }");
        self.write_line("    snprintf(buf + pos, 2, \"]\");");
        self.write_line("    return buf;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline char* _ox_to_str_str_slice(_OxStrSlice s) {");
        self.write_line("    char* buf = _ox_arena_alloc(s.len + 1);");
        self.write_line("    if (s.ptr && s.len > 0) {");
        self.write_line("        memcpy(buf, s.ptr, s.len);");
        self.write_line("    }");
        self.write_line("    buf[s.len] = '\\0';");
        self.write_line("    return buf;");
        self.write_line("}");
        self.write_line("");
        self.write_line("typedef int64_t (*_OxClosure1)(int64_t);");
        self.write_line("static inline int64_t* _ox_arr_map(int64_t* arr, _OxClosure1 fn) {");
        self.write_line("    if (!arr) return _ox_array_create(0);");
        self.write_line("    int64_t count = _OX_ARRAY_HDR(arr)->len;");
        self.write_line("    int64_t* res = _ox_array_create(count);");
        self.write_line("    for (int64_t i = 0; i < count; i++) {");
        self.write_line("        res[i] = fn(arr[i]);");
        self.write_line("    }");
        self.write_line("    return res;");
        self.write_line("}");
        self.write_line("");
        self.write_line("typedef int64_t (*_OxClosurePred)(int64_t);");
        self.write_line("static inline int64_t* _ox_arr_filter(int64_t* arr, _OxClosurePred fn) {");
        self.write_line("    if (!arr) return _ox_array_create(0);");
        self.write_line("    int64_t count = _OX_ARRAY_HDR(arr)->len;");
        self.write_line("    int64_t* res = _ox_array_create(count);");
        self.write_line("    int64_t cur = 0;");
        self.write_line("    for (int64_t i = 0; i < count; i++) {");
        self.write_line("        if (fn(arr[i])) {");
        self.write_line("            res[cur++] = arr[i];");
        self.write_line("        }");
        self.write_line("    }");
        self.write_line("    _OX_ARRAY_HDR(res)->len = cur;");
        self.write_line("    return res;");
        self.write_line("}");
        self.write_line("");
        self.write_line("#define arena_reset() (_ox_arena_pos = 0)");
        self.write_line("");
        self.write_line("static inline const char* _ox_str_trim(const char* s) {");
        self.write_line("    if (!s) return \"\";");
        self.write_line("    while (*s && (*s == ' ' || *s == '\\t' || *s == '\\n' || *s == '\\r')) s++;");
        self.write_line("    if (!*s) return \"\";");
        self.write_line("    size_t len = strlen(s);");
        self.write_line("    while (len > 0 && (s[len - 1] == ' ' || s[len - 1] == '\\t' || s[len - 1] == '\\n' || s[len - 1] == '\\r')) len--;");
        self.write_line("    char* res = _ox_arena_alloc(len + 1);");
        self.write_line("    memcpy(res, s, len);");
        self.write_line("    res[len] = '\\0';");
        self.write_line("    return res;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline const char* _ox_str_to_upper(const char* s) {");
        self.write_line("    if (!s) return \"\";");
        self.write_line("    size_t len = strlen(s);");
        self.write_line("    char* res = _ox_arena_alloc(len + 1);");
        self.write_line("    for (size_t i = 0; i < len; i++) {");
        self.write_line("        char c = s[i];");
        self.write_line("        if (c >= 'a' && c <= 'z') c -= ('a' - 'A');");
        self.write_line("        res[i] = c;");
        self.write_line("    }");
        self.write_line("    res[len] = '\\0';");
        self.write_line("    return res;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline const char* _ox_str_to_lower(const char* s) {");
        self.write_line("    if (!s) return \"\";");
        self.write_line("    size_t len = strlen(s);");
        self.write_line("    char* res = _ox_arena_alloc(len + 1);");
        self.write_line("    for (size_t i = 0; i < len; i++) {");
        self.write_line("        char c = s[i];");
        self.write_line("        if (c >= 'A' && c <= 'Z') c += ('a' - 'A');");
        self.write_line("        res[i] = c;");
        self.write_line("    }");
        self.write_line("    res[len] = '\\0';");
        self.write_line("    return res;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline bool _ox_str_contains(const char* s, const char* sub) {");
        self.write_line("    if (!s || !sub) return false;");
        self.write_line("    return strstr(s, sub) != NULL;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline char _ox_char_at(const char* s, int64_t idx) {");
        self.write_line("    if (!s || idx < 0) return '\\0';");
        self.write_line("    size_t len = strlen(s);");
        self.write_line("    if ((size_t)idx >= len) return '\\0';");
        self.write_line("    return s[idx];");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline const char** _ox_str_array_create(int64_t count) {");
        self.write_line("    int64_t cap = count < 8 ? 8 : count * 2;");
        self.write_line("    size_t total = sizeof(_OxArrayHeader) + cap * sizeof(const char*);");
        self.write_line("    _OxArrayHeader* hdr = (_OxArrayHeader*)_ox_arena_alloc(total);");
        self.write_line("    hdr->len = count;");
        self.write_line("    hdr->cap = cap;");
        self.write_line("    return (const char**)(hdr + 1);");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline const char** _ox_str_split(const char* s, const char* sep) {");
        self.write_line("    if (!s) s = \"\";");
        self.write_line("    if (!sep || !*sep) sep = \" \";");
        self.write_line("    size_t sep_len = strlen(sep);");
        self.write_line("    int64_t count = 0;");
        self.write_line("    const char* cur = s;");
        self.write_line("    while ((cur = strstr(cur, sep)) != NULL) {");
        self.write_line("        count++;");
        self.write_line("        cur += sep_len;");
        self.write_line("    }");
        self.write_line("    count++;");
        self.write_line("    const char** arr = _ox_str_array_create(count);");
        self.write_line("    cur = s;");
        self.write_line("    int64_t idx = 0;");
        self.write_line("    const char* next_match;");
        self.write_line("    while ((next_match = strstr(cur, sep)) != NULL) {");
        self.write_line("        size_t part_len = next_match - cur;");
        self.write_line("        char* part = _ox_arena_alloc(part_len + 1);");
        self.write_line("        memcpy(part, cur, part_len);");
        self.write_line("        part[part_len] = '\\0';");
        self.write_line("        arr[idx++] = (const char*)part;");
        self.write_line("        cur = next_match + sep_len;");
        self.write_line("    }");
        self.write_line("    size_t rest_len = strlen(cur);");
        self.write_line("    char* rest = _ox_arena_alloc(rest_len + 1);");
        self.write_line("    memcpy(rest, cur, rest_len);");
        self.write_line("    rest[rest_len] = '\\0';");
        self.write_line("    arr[idx++] = (const char*)rest;");
        self.write_line("    return arr;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline const char* _ox_str_replace(const char* s, const char* from, const char* to) {");
        self.write_line("    if (!s) return \"\";");
        self.write_line("    if (!from || !*from) return s;");
        self.write_line("    if (!to) to = \"\";");
        self.write_line("    size_t from_len = strlen(from);");
        self.write_line("    size_t to_len = strlen(to);");
        self.write_line("    size_t count = 0;");
        self.write_line("    const char* p = s;");
        self.write_line("    while ((p = strstr(p, from)) != NULL) {");
        self.write_line("        count++;");
        self.write_line("        p += from_len;");
        self.write_line("    }");
        self.write_line("    if (count == 0) return s;");
        self.write_line("    size_t new_len = strlen(s) + count * (to_len > from_len ? (to_len - from_len) : 0);");
        self.write_line("    char* res = _ox_arena_alloc(new_len + 1);");
        self.write_line("    char* out = res;");
        self.write_line("    p = s;");
        self.write_line("    while (*p) {");
        self.write_line("        if (strncmp(p, from, from_len) == 0) {");
        self.write_line("            memcpy(out, to, to_len);");
        self.write_line("            out += to_len;");
        self.write_line("            p += from_len;");
        self.write_line("        } else {");
        self.write_line("            *out++ = *p++;");
        self.write_line("        }");
        self.write_line("    }");
        self.write_line("    *out = '\\0';");
        self.write_line("    return res;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static int _ox_argc = 0;");
        self.write_line("static char** _ox_argv = NULL;");
        self.write_line("");
        self.write_line("static inline const char** _ox_args(void) {");
        self.write_line("    int64_t count = _ox_argc;");
        self.write_line("    const char** arr = _ox_str_array_create(count);");
        self.write_line("    for (int64_t i = 0; i < count; i++) {");
        self.write_line("        arr[i] = _ox_argv[i];");
        self.write_line("    }");
        self.write_line("    return arr;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline const char* _ox_get_env(const char* key) {");
        self.write_line("    if (!key) return \"\";");
        self.write_line("    const char* val = getenv(key);");
        self.write_line("    return val ? val : \"\";");
        self.write_line("}");
        self.write_line("");
        self.write_line("typedef struct _OxMapEntry {");
        self.write_line("    const char* key;");
        self.write_line("    const char* value;");
        self.write_line("    struct _OxMapEntry* next;");
        self.write_line("} _OxMapEntry;");
        self.write_line("");
        self.write_line("#define _OX_MAP_BUCKETS 64");
        self.write_line("");
        self.write_line("typedef struct {");
        self.write_line("    _OxMapEntry* buckets[_OX_MAP_BUCKETS];");
        self.write_line("    int64_t count;");
        self.write_line("} _OxMap;");
        self.write_line("");
        self.write_line("static inline uint32_t _ox_hash_str(const char* str) {");
        self.write_line("    uint32_t hash = 5381;");
        self.write_line("    int c;");
        self.write_line("    if (!str) return 0;");
        self.write_line("    while ((c = *str++)) hash = ((hash << 5) + hash) + c;");
        self.write_line("    return hash;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline _OxMap* _ox_map_new(void) {");
        self.write_line("    _OxMap* m = (_OxMap*)_ox_arena_alloc(sizeof(_OxMap));");
        self.write_line("    memset(m, 0, sizeof(_OxMap));");
        self.write_line("    return m;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline void _ox_map_set(_OxMap* m, const char* key, const char* val) {");
        self.write_line("    if (!m || !key) return;");
        self.write_line("    uint32_t idx = _ox_hash_str(key) % _OX_MAP_BUCKETS;");
        self.write_line("    _OxMapEntry* entry = m->buckets[idx];");
        self.write_line("    while (entry) {");
        self.write_line("        if (strcmp(entry->key, key) == 0) {");
        self.write_line("            entry->value = val;");
        self.write_line("            return;");
        self.write_line("        }");
        self.write_line("        entry = entry->next;");
        self.write_line("    }");
        self.write_line("    _OxMapEntry* new_entry = (_OxMapEntry*)_ox_arena_alloc(sizeof(_OxMapEntry));");
        self.write_line("    new_entry->key = key;");
        self.write_line("    new_entry->value = val;");
        self.write_line("    new_entry->next = m->buckets[idx];");
        self.write_line("    m->buckets[idx] = new_entry;");
        self.write_line("    m->count++;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline const char* _ox_map_get(_OxMap* m, const char* key) {");
        self.write_line("    if (!m || !key) return \"\";");
        self.write_line("    uint32_t idx = _ox_hash_str(key) % _OX_MAP_BUCKETS;");
        self.write_line("    _OxMapEntry* entry = m->buckets[idx];");
        self.write_line("    while (entry) {");
        self.write_line("        if (strcmp(entry->key, key) == 0) return entry->value ? entry->value : \"\";");
        self.write_line("        entry = entry->next;");
        self.write_line("    }");
        self.write_line("    return \"\";");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline bool _ox_map_has(_OxMap* m, const char* key) {");
        self.write_line("    if (!m || !key) return false;");
        self.write_line("    uint32_t idx = _ox_hash_str(key) % _OX_MAP_BUCKETS;");
        self.write_line("    _OxMapEntry* entry = m->buckets[idx];");
        self.write_line("    while (entry) {");
        self.write_line("        if (strcmp(entry->key, key) == 0) return true;");
        self.write_line("        entry = entry->next;");
        self.write_line("    }");
        self.write_line("    return false;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline int64_t _ox_map_len(_OxMap* m) {");
        self.write_line("    if (!m) return 0LL;");
        self.write_line("    return m->count;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline const char** _ox_map_keys(_OxMap* m) {");
        self.write_line("    int64_t count = m ? m->count : 0;");
        self.write_line("    const char** arr = _ox_str_array_create(count);");
        self.write_line("    if (!m) return arr;");
        self.write_line("    int64_t cur = 0;");
        self.write_line("    for (int i = 0; i < _OX_MAP_BUCKETS; i++) {");
        self.write_line("        _OxMapEntry* entry = m->buckets[i];");
        self.write_line("        while (entry) {");
        self.write_line("            arr[cur++] = entry->key;");
        self.write_line("            entry = entry->next;");
        self.write_line("        }");
        self.write_line("    }");
        self.write_line("    return arr;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline void _ox_print_str_array(const char** arr) {");
        self.write_line("    if (!arr) { printf(\"[]\"); return; }");
        self.write_line("    int64_t len = _OX_ARRAY_HDR(arr)->len;");
        self.write_line("    printf(\"[\");");
        self.write_line("    for (int64_t i = 0; i < len; i++) {");
        self.write_line("        if (i > 0) printf(\", \");");
        self.write_line("        printf(\"\\\"%s\\\"\", arr[i] ? arr[i] : \"\");");
        self.write_line("    }");
        self.write_line("    printf(\"]\");");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline void _ox_print_map(_OxMap* m) {");
        self.write_line("    if (!m) { printf(\"Map {}\"); return; }");
        self.write_line("    printf(\"Map { \");");
        self.write_line("    int64_t printed = 0;");
        self.write_line("    for (int i = 0; i < _OX_MAP_BUCKETS; i++) {");
        self.write_line("        _OxMapEntry* entry = m->buckets[i];");
        self.write_line("        while (entry) {");
        self.write_line("            if (printed > 0) printf(\", \");");
        self.write_line("            printf(\"\\\"%s\\\": %s\", entry->key, entry->value ? entry->value : \"\\\"\\\"\");");
        self.write_line("            printed++;");
        self.write_line("            entry = entry->next;");
        self.write_line("        }");
        self.write_line("    }");
        self.write_line("    printf(\" }\");");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline char* _ox_to_str_map(_OxMap* m) {");
        self.write_line("    char* buf = _ox_arena_alloc(256);");
        self.write_line("    snprintf(buf, 256, \"Map { len: %ld }\", (long)(m ? m->count : 0));");
        self.write_line("    return buf;");
        self.write_line("}");
        self.write_line("");
        self.write_line("#define len(x) _Generic((x), \\");
        self.write_line("    _OxSlice: _ox_slice_len, \\");
        self.write_line("    _OxStrSlice: _ox_str_slice_len, \\");
        self.write_line("    char*: _ox_str_len, \\");
        self.write_line("    const char*: _ox_str_len, \\");
        self.write_line("    _OxMap*: _ox_map_len, \\");
        self.write_line("    default: _ox_arr_len \\");
        self.write_line(")(x)");
        self.write_line("");
        self.write_line("#define push(arr, val) ((arr) = _ox_arr_push_impl((arr), (int64_t)(val)))");
        self.write_line("#define pop(arr) (_OX_ARRAY_HDR(arr)->len > 0 ? (arr)[--_OX_ARRAY_HDR(arr)->len] : 0LL)");
        self.write_line("");
        self.write_line("static inline const char* _ox_read_file(const char* path) {");
        self.write_line("    if (!path) return \"\";");
        self.write_line("    FILE* f = fopen(path, \"rb\");");
        self.write_line("    if (!f) return \"\";");
        self.write_line("    fseek(f, 0, SEEK_END);");
        self.write_line("    long sz = ftell(f);");
        self.write_line("    fseek(f, 0, SEEK_SET);");
        self.write_line("    if (sz < 0) { fclose(f); return \"\"; }");
        self.write_line("    char* buf = _ox_arena_alloc(sz + 1);");
        self.write_line("    size_t read_bytes = fread(buf, 1, sz, f);");
        self.write_line("    buf[read_bytes] = '\\0';");
        self.write_line("    fclose(f);");
        self.write_line("    return buf;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline bool _ox_write_file(const char* path, const char* content) {");
        self.write_line("    if (!path) return false;");
        self.write_line("    FILE* f = fopen(path, \"wb\");");
        self.write_line("    if (!f) return false;");
        self.write_line("    if (content) {");
        self.write_line("        fputs(content, f);");
        self.write_line("    }");
        self.write_line("    fclose(f);");
        self.write_line("    return true;");
        self.write_line("}");
        self.write_line("");
        self.write_line("static inline const char* _ox_read_line(void) {");
        self.write_line("    char temp[4096];");
        self.write_line("    if (!fgets(temp, sizeof(temp), stdin)) {");
        self.write_line("        return \"\";");
        self.write_line("    }");
        self.write_line("    size_t len = strlen(temp);");
        self.write_line("    while (len > 0 && (temp[len - 1] == '\\n' || temp[len - 1] == '\\r')) {");
        self.write_line("        temp[--len] = '\\0';");
        self.write_line("    }");
        self.write_line("    char* buf = _ox_arena_alloc(len + 1);");
        self.write_line("    memcpy(buf, temp, len + 1);");
        self.write_line("    return buf;");
        self.write_line("}");
        self.write_line("");
        self.write_line("#define read_file(p) _ox_read_file(p)");
        self.write_line("#define write_file(p, c) _ox_write_file(p, c)");
        self.write_line("#define read_line() _ox_read_line()");
        self.write_line("#define to_int(x) ((int64_t)(x))");
        self.write_line("#define to_float(x) ((double)(x))");
        self.write_line("#define to_char(x) ((char)(x))");
        self.write_line("#define to_u8(x) ((uint8_t)(x))");
        self.write_line("#define to_byte(x) ((uint8_t)(x))");
        self.write_line("#ifndef abs");
        self.write_line("#define abs(x) llabs(x)");
        self.write_line("#endif");
        self.write_line("");
        self.write_line("typedef struct { bool is_some; int64_t val; } _OxOption;");
        self.write_line("typedef struct { bool is_ok; int64_t ok; int64_t err; } _OxResult;");
        self.write_line("typedef struct _OxRc { int64_t count; void* val; } _OxRc;");
        self.write_line("static inline _OxRc* _ox_rc_clone(_OxRc* r) { if (r) r->count++; return r; }");
        self.write_line("static inline _OxOption _ox_some(int64_t v) { return (_OxOption){ .is_some = true, .val = v }; }");
        self.write_line("static inline _OxOption _ox_none(void) { return (_OxOption){ .is_some = false, .val = 0 }; }");
        self.write_line("static inline _OxResult _ox_ok(int64_t v) { return (_OxResult){ .is_ok = true, .ok = v, .err = 0 }; }");
        self.write_line("static inline _OxResult _ox_err(int64_t e) { return (_OxResult){ .is_ok = false, .ok = 0, .err = e }; }");
        self.write_line("static inline void _ox_panic(const char* msg) { fprintf(stderr, \"Panik: %s\\n\", msg); exit(1); }");
        self.write_line("static inline int64_t _ox_option_unwrap(_OxOption opt) { if (!opt.is_some) _ox_panic(\"called unwrap() on a None value\"); return opt.val; }");
        self.write_line("static inline int64_t _ox_result_unwrap(_OxResult res) { if (!res.is_ok) _ox_panic(\"called unwrap() on an Err value\"); return res.ok; }");
        self.write_line("static inline int64_t _ox_option_unwrap_or(_OxOption opt, int64_t def) { return opt.is_some ? opt.val : def; }");
        self.write_line("static inline int64_t _ox_result_unwrap_or(_OxResult res, int64_t def) { return res.is_ok ? res.ok : def; }");
        self.write_line("#define _ox_unwrap(x) _Generic((x), _OxOption: _ox_option_unwrap, _OxResult: _ox_result_unwrap)(x)");
        self.write_line("#define _ox_unwrap_or(x, def) _Generic((x), _OxOption: _ox_option_unwrap_or, _OxResult: _ox_result_unwrap_or)((x), (int64_t)(def))");
        self.write_line("static inline void _ox_print_option(_OxOption opt) { if (opt.is_some) { printf(\"Some(\"); _ox_print_int(opt.val); printf(\")\"); } else { printf(\"None\"); } }");
        self.write_line("static inline void _ox_print_result(_OxResult res) { if (res.is_ok) { printf(\"Ok(\"); _ox_print_int(res.ok); printf(\")\"); } else { printf(\"Err(\"); _ox_print_str((const char*)res.err); printf(\")\"); } }");
        self.write_line("static inline char* _ox_to_str_option(_OxOption opt) { if (opt.is_some) { char* buf = _ox_arena_alloc(64); snprintf(buf, 64, \"Some(%ld)\", (long)opt.val); return buf; } else { return \"None\"; } }");
        self.write_line("static inline char* _ox_to_str_result(_OxResult res) { if (res.is_ok) { char* buf = _ox_arena_alloc(64); snprintf(buf, 64, \"Ok(%ld)\", (long)res.ok); return buf; } else { char* buf = _ox_arena_alloc(64); snprintf(buf, 64, \"Err(%s)\", (const char*)res.err); return buf; } }");
        self.write_line("");

        // Tuple type declarations
        for tuple_tys in &self.tuples.clone() {
            let tuple_name = self.c_tuple_name(tuple_tys);
            self.write_line(&format!("typedef struct {{"));
            self.indent += 1;
            for (idx, elem_t) in tuple_tys.iter().enumerate() {
                self.write_line(&format!("{} _{};", self.to_c_type(elem_t), idx));
            }
            self.indent -= 1;
            self.write_line(&format!("}} {};", tuple_name));
            self.write_line(&format!("static inline void _ox_print_{}({} t);", tuple_name, tuple_name));
            self.write_line(&format!("static inline char* _ox_to_str_{}({} t);", tuple_name, tuple_name));
        }
        self.write_line("");

        let tuples = self.tuples.clone();

        self.write_line("#define _ox_print_item(x) _Generic((x), \\");
        for tuple_tys in &tuples {
            let tuple_name = self.c_tuple_name(tuple_tys);
            self.write_line(&format!("    {}: _ox_print_{}, \\", tuple_name, tuple_name));
        }
        self.write_line("    _OxOption: _ox_print_option, \\");
        self.write_line("    _OxResult: _ox_print_result, \\");
        self.write_line("    _OxSlice: _ox_print_slice, \\");
        self.write_line("    _OxStrSlice: _ox_print_str_slice, \\");
        self.write_line("    int: _ox_print_int, \\");
        self.write_line("    long: _ox_print_int, \\");
        self.write_line("    long long: _ox_print_int, \\");
        self.write_line("    double: _ox_print_double, \\");
        self.write_line("    float: _ox_print_double, \\");
        self.write_line("    bool: _ox_print_bool, \\");
        self.write_line("    char: _ox_print_char, \\");
        self.write_line("    unsigned char: _ox_print_u8, \\");
        self.write_line("    char*: _ox_print_str, \\");
        self.write_line("    const char*: _ox_print_str, \\");
        self.write_line("    int64_t*: _ox_print_array, \\");
        self.write_line("    const char**: _ox_print_str_array, \\");
        self.write_line("    char**: _ox_print_str_array, \\");
        self.write_line("    _OxMap*: _ox_print_map, \\");
        self.write_line("    default: _ox_print_ptr \\");
        self.write_line(")(x)");
        self.write_line("");

        for tuple_tys in &tuples {
            let tuple_name = self.c_tuple_name(tuple_tys);
            self.write_line(&format!("static inline void _ox_print_{}({} t) {{", tuple_name, tuple_name));
            self.write_line("    printf(\"(\");");
            for (idx, _) in tuple_tys.iter().enumerate() {
                if idx > 0 {
                    self.write_line("    printf(\", \");");
                }
                self.write_line(&format!("    _ox_print_item(t._{});", idx));
            }
            self.write_line("    printf(\")\");");
            self.write_line("}");
        }
        self.write_line("");
        self.write_line("#define _ox_to_str(x) _Generic((x), \\");
        for tuple_tys in &tuples {
            let tuple_name = self.c_tuple_name(tuple_tys);
            self.write_line(&format!("    {}: _ox_to_str_{}, \\", tuple_name, tuple_name));
        }
        self.write_line("    _OxOption: _ox_to_str_option, \\");
        self.write_line("    _OxResult: _ox_to_str_result, \\");
        self.write_line("    _OxSlice: _ox_to_str_slice, \\");
        self.write_line("    _OxStrSlice: _ox_to_str_str_slice, \\");
        self.write_line("    int: _ox_to_str_int, \\");
        self.write_line("    long: _ox_to_str_int, \\");
        self.write_line("    long long: _ox_to_str_int, \\");
        self.write_line("    double: _ox_to_str_double, \\");
        self.write_line("    float: _ox_to_str_double, \\");
        self.write_line("    bool: _ox_to_str_bool, \\");
        self.write_line("    char: _ox_to_str_char, \\");
        self.write_line("    unsigned char: _ox_to_str_u8, \\");
        self.write_line("    char*: _ox_to_str_str, \\");
        self.write_line("    const char*: _ox_to_str_str, \\");
        self.write_line("    _OxMap*: _ox_to_str_map, \\");
        self.write_line("    default: _ox_to_str_str \\");
        self.write_line(")(x)");
        self.write_line("");

        for tuple_tys in &tuples {
            let tuple_name = self.c_tuple_name(tuple_tys);
            self.write_line(&format!("static inline char* _ox_to_str_{}({} t) {{", tuple_name, tuple_name));
            self.write_line("    char* res = \"(\";");
            for (idx, _) in tuple_tys.iter().enumerate() {
                if idx > 0 {
                    self.write_line("    res = _ox_str_concat(res, \", \");");
                }
                self.write_line(&format!("    res = _ox_str_concat(res, _ox_to_str(t._{}));", idx));
            }
            self.write_line("    res = _ox_str_concat(res, \")\");");
            self.write_line("    return res;");
            self.write_line("}");
        }
        self.write_line("");
        self.write_line("#define _ox_add_str(a, b) _ox_str_concat(_ox_to_str(a), _ox_to_str(b))");
        self.write_line("");

        // Enums definition
        for item in &program.items {
            if let Item::Enum(e) = item {
                self.compile_enum(e);
            }
        }

        // Structs forward declarations
        self.write_line("/* Struct Forward Declarations */");
        for item in &program.items {
            if let Item::Struct(s) = item {
                self.write_line(&format!("typedef struct {} {};", s.name, s.name));
            }
        }
        self.write_line("");

        // Struct definitions
        for item in &program.items {
            if let Item::Struct(s) = item {
                self.compile_struct(s);
            }
        }

        // Function prototypes
        self.write_line("/* Function Prototypes */");
        for item in &program.items {
            match item {
                Item::Function(f) => {
                    let proto = self.compile_fn_prototype(f, None);
                    self.write_line(&format!("{};", proto));
                }
                Item::Impl(imp) => {
                    for m in &imp.methods {
                        let proto = self.compile_fn_prototype(m, Some(&imp.target));
                        self.write_line(&format!("{};", proto));
                    }
                }
                Item::Extern(ext) => {
                    self.write_line(&format!("/* Extern \"{}\" Prototypes */", ext.abi));
                    for f in &ext.functions {
                        if is_libc_builtin(&f.name) {
                            continue;
                        }
                        let ret_ty = self.to_c_type(&f.return_type);
                        let param_strs: Vec<String> = f.params.iter().map(|p| format!("{} {}", self.to_c_type(&p.ty), p.name)).collect();
                        let params_joined = if param_strs.is_empty() { "void".to_string() } else { param_strs.join(", ") };
                        self.write_line(&format!("extern {} {}({});", ret_ty, f.name, params_joined));
                    }
                }
                _ => {}
            }
        }
        self.write_line("");

        // Function & Method Implementations
        self.write_line("/* Implementations */");
        for item in &program.items {
            match item {
                Item::Function(f) => {
                    self.compile_function(f, None);
                }
                Item::Impl(imp) => {
                    for m in &imp.methods {
                        self.compile_function(m, Some(&imp.target));
                    }
                }
                _ => {}
            }
        }

        self.output
    }

    fn compile_enum(&mut self, e: &EnumDef) {
        self.write_line(&format!("typedef enum {} {{", e.name));
        self.indent += 1;
        for variant in &e.variants {
            self.write_line(&format!("{}_{},", e.name, variant));
        }
        self.indent -= 1;
        self.write_line(&format!("}} {};", e.name));
        self.write_line("");
    }

    fn compile_struct(&mut self, s: &StructDef) {
        self.write_line(&format!("struct {} {{", s.name));
        self.indent += 1;
        for (f_name, f_ty) in &s.fields {
            let c_ty = self.to_c_type(f_ty);
            self.write_line(&format!("{} {};", c_ty, f_name));
        }
        self.indent -= 1;
        self.write_line("};");
        self.write_line("");
    }

    fn compile_fn_prototype(&self, f: &Function, impl_target: Option<&str>) -> String {
        let fn_name = if let Some(target) = impl_target {
            format!("{}_{}", target, f.name)
        } else {
            f.name.clone()
        };

        let ret_ty = self.to_c_type(&f.return_type);

        let mut param_strs = Vec::new();
        for param in &f.params {
            let ty_str = if param.name == "self" {
                if let Some(target) = impl_target {
                    format!("{}*", target)
                } else {
                    self.to_c_type(&param.ty)
                }
            } else {
                self.to_c_type(&param.ty)
            };
            param_strs.push(format!("{} {}", ty_str, param.name));
        }

        let params_joined = if param_strs.is_empty() {
            "void".to_string()
        } else {
            param_strs.join(", ")
        };

        if f.name == "main" && impl_target.is_none() {
            "int main(int argc, char** argv)".to_string()
        } else {
            format!("{} {}({})", ret_ty, fn_name, params_joined)
        }
    }

    fn compile_function(&mut self, f: &Function, impl_target: Option<&str>) {
        self.current_impl = impl_target.map(|s| s.to_string());
        self.current_fn_ret = Some(f.return_type.clone());
        self.var_types.clear();
        self.var_ox_types.clear();
        for param in &f.params {
            let actual_param_ty = if param.name == "self" {
                if let Some(target) = impl_target {
                    Type::Named(target.to_string())
                } else {
                    param.ty.clone()
                }
            } else {
                param.ty.clone()
            };
            self.var_ox_types.insert(param.name.clone(), actual_param_ty.clone());
            if let Type::Named(s) = &actual_param_ty {
                self.var_types.insert(param.name.clone(), s.clone());
            }
        }
        let proto = self.compile_fn_prototype(f, impl_target);
        self.write_line(&format!("{} {{", proto));
        self.indent += 1;

        if f.name == "main" && impl_target.is_none() {
            self.write_line("_ox_argc = argc;");
            self.write_line("_ox_argv = argv;");
        }

        // Compile body block
        for stmt in &f.body.stmts {
            self.compile_stmt(stmt);
        }

        if let Some(trailing) = &f.body.trailing_expr {
            let expr_str = self.compile_expr(trailing);
            if f.name == "main" && impl_target.is_none() {
                self.write_line(&format!("{};", expr_str));
                self.write_line("return 0;");
            } else if f.return_type == Type::Void {
                self.write_line(&format!("{};", expr_str));
            } else {
                self.write_line(&format!("return {};", expr_str));
            }
        } else if f.name == "main" && impl_target.is_none() {
            self.write_line("return 0;");
        }

        self.indent -= 1;
        self.write_line("}");
        self.write_line("");
        self.current_fn_ret = None;
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, ty, init, .. } => {
                let actual_ty = if let Some(t) = ty {
                    t.clone()
                } else if let Some(e) = init {
                    self.infer_expr_type(e)
                } else {
                    Type::Int
                };
                self.var_ox_types.insert(name.clone(), actual_ty.clone());
                if let Type::Named(s) = &actual_ty {
                    self.var_types.insert(name.clone(), s.clone());
                } else if let Some(Expr::StructInit { name: s, .. }) = init {
                    self.var_types.insert(name.clone(), s.clone());
                }

                let type_str = if let Some(t) = ty {
                    self.to_c_type(t)
                } else {
                    "auto".to_string() // GCC supports __auto_type / auto
                };

                let c_type = if type_str == "auto" { "__auto_type" } else { &type_str };

                if let Some(expr) = init {
                    let val = self.compile_expr(expr);
                    self.write_line(&format!("{} {} = {};", c_type, name, val));
                } else {
                    self.write_line(&format!("{} {};", c_type, name));
                }
            }
            Stmt::LetTuple { elements, ty, init, .. } => {
                let inferred_ty = init.as_ref().map(|e| self.infer_expr_type(e));
                let tuple_ty = ty.clone().or(inferred_ty);
                let elem_tys = match &tuple_ty {
                    Some(Type::Tuple(tys)) => Some(tys.clone()),
                    _ => None,
                };
                if let Some(ref tys) = elem_tys {
                    for (i, (name, _)) in elements.iter().enumerate() {
                        if name != "_" {
                            if let Some(t) = tys.get(i) {
                                self.var_ox_types.insert(name.clone(), t.clone());
                                if let Type::Named(s) = t {
                                    self.var_types.insert(name.clone(), s.clone());
                                }
                            }
                        }
                    }
                }
                if let Some(expr) = init {
                    let val = self.compile_expr(expr);
                    self.temp_counter += 1;
                    let tmp_var = format!("_ox_tuple_tmp_{}", self.temp_counter);
                    let c_type = if let Some(t) = ty {
                        self.to_c_type(t)
                    } else if let Some(ref tys) = elem_tys {
                        self.c_tuple_name(tys)
                    } else {
                        "__auto_type".to_string()
                    };
                    self.write_line(&format!("{} {} = {};", c_type, tmp_var, val));
                    for (i, (name, _)) in elements.iter().enumerate() {
                        if name != "_" {
                            self.write_line(&format!("__auto_type {} = {}._{};", name, tmp_var, i));
                        }
                    }
                } else {
                    for (i, (name, _)) in elements.iter().enumerate() {
                        if name != "_" {
                            let c_type = if let Some(ref tys) = elem_tys {
                                self.to_c_type(&tys[i])
                            } else {
                                "int64_t".to_string()
                            };
                            self.write_line(&format!("{} {};", c_type, name));
                        }
                    }
                }
            }
            Stmt::While { condition, body, .. } => {
                let cond_str = self.compile_expr(condition);
                self.write_line(&format!("while ({}) {{", cond_str));
                self.indent += 1;
                for s in &body.stmts {
                    self.compile_stmt(s);
                }
                if let Some(t) = &body.trailing_expr {
                    let expr_str = self.compile_expr(t);
                    self.write_line(&format!("{};", expr_str));
                }
                self.indent -= 1;
                self.write_line("}");
            }
            Stmt::ForRange { var_name, start, end, body, .. } => {
                let start_str = self.compile_expr(start);
                let end_str = self.compile_expr(end);
                self.write_line(&format!("for (int64_t {} = {}; {} < {}; {}++) {{", var_name, start_str, var_name, end_str, var_name));
                self.indent += 1;
                for s in &body.stmts {
                    self.compile_stmt(s);
                }
                if let Some(t) = &body.trailing_expr {
                    let expr_str = self.compile_expr(t);
                    self.write_line(&format!("{};", expr_str));
                }
                self.indent -= 1;
                self.write_line("}");
            }
            Stmt::ForIn { var_name, iterable, body, .. } => {
                let iter_str = self.compile_expr(iterable);
                let iter_ty = self.infer_expr_type(iterable);
                let elem_ty = match &iter_ty {
                    Type::Array(inner) | Type::Slice(inner) => (**inner).clone(),
                    Type::Str | Type::StrSlice => Type::Char,
                    _ => Type::Int,
                };
                let old_ox = self.var_ox_types.insert(var_name.clone(), elem_ty.clone());
                let old_t = if let Type::Named(ref s) = elem_ty {
                    self.var_types.insert(var_name.clone(), s.clone())
                } else {
                    None
                };

                self.write_line("{");
                self.indent += 1;
                self.write_line(&format!("__auto_type _iter = {};", iter_str));
                match iter_ty {
                    Type::Slice(..) => {
                        self.write_line("int64_t _len = _iter.len;");
                        self.write_line("for (int64_t _i = 0; _i < _len; _i++) {");
                        self.indent += 1;
                        self.write_line(&format!("__auto_type {} = _iter.ptr[_i];", var_name));
                    }
                    Type::StrSlice => {
                        self.write_line("int64_t _len = _iter.len;");
                        self.write_line("for (int64_t _i = 0; _i < _len; _i++) {");
                        self.indent += 1;
                        self.write_line(&format!("char {} = _iter.ptr[_i];", var_name));
                    }
                    Type::Str => {
                        self.write_line("int64_t _len = _ox_str_len(_iter);");
                        self.write_line("for (int64_t _i = 0; _i < _len; _i++) {");
                        self.indent += 1;
                        self.write_line(&format!("char {} = _iter[_i];", var_name));
                    }
                    _ => {
                        self.write_line("int64_t _len = _ox_arr_len(_iter);");
                        self.write_line("for (int64_t _i = 0; _i < _len; _i++) {");
                        self.indent += 1;
                        self.write_line(&format!("__auto_type {} = _iter[_i];", var_name));
                    }
                }
                for s in &body.stmts {
                    self.compile_stmt(s);
                }
                if let Some(t) = &body.trailing_expr {
                    let expr_str = self.compile_expr(t);
                    self.write_line(&format!("{};", expr_str));
                }
                self.indent -= 1;
                self.write_line("}");
                self.indent -= 1;
                self.write_line("}");

                if let Some(prev) = old_ox {
                    self.var_ox_types.insert(var_name.clone(), prev);
                } else {
                    self.var_ox_types.remove(var_name);
                }
                if let Some(prev) = old_t {
                    self.var_types.insert(var_name.clone(), prev);
                } else {
                    self.var_types.remove(var_name);
                }
            }
            Stmt::Break(_) => {
                self.write_line("break;");
            }
            Stmt::Continue(_) => {
                self.write_line("continue;");
            }
            Stmt::Return(maybe_expr, _) => {
                if let Some(expr) = maybe_expr {
                    let val = self.compile_expr(expr);
                    self.write_line(&format!("return {};", val));
                } else {
                    self.write_line("return;");
                }
            }
            Stmt::Expr(Expr::If { condition, then_branch, else_branch, .. }) => {
                let cond_str = self.compile_expr(condition);
                self.write_line(&format!("if ({}) {{", cond_str));
                self.indent += 1;
                for s in &then_branch.stmts {
                    self.compile_stmt(s);
                }
                if let Some(t) = &then_branch.trailing_expr {
                    let expr_str = self.compile_expr(t);
                    self.write_line(&format!("{};", expr_str));
                }
                self.indent -= 1;
                if let Some(eb) = else_branch {
                    self.write_line("} else {");
                    self.indent += 1;
                    for s in &eb.stmts {
                        self.compile_stmt(s);
                    }
                    if let Some(t) = &eb.trailing_expr {
                        let expr_str = self.compile_expr(t);
                        self.write_line(&format!("{};", expr_str));
                    }
                    self.indent -= 1;
                }
                self.write_line("}");
            }
            Stmt::Expr(Expr::Block(block)) | Stmt::Expr(Expr::UnsafeBlock(block, _)) => {
                self.write_line("{");
                self.indent += 1;
                for s in &block.stmts {
                    self.compile_stmt(s);
                }
                if let Some(t) = &block.trailing_expr {
                    let expr_str = self.compile_expr(t);
                    self.write_line(&format!("{};", expr_str));
                }
                self.indent -= 1;
                self.write_line("}");
            }
            Stmt::Expr(expr) => {
                let expr_str = self.compile_expr(expr);
                self.write_line(&format!("{};", expr_str));
            }
        }
    }

    fn compile_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(i, _) => format!("{}LL", i),
            Expr::Float(f, _) => format!("{}", f),
            Expr::Bool(b, _) => if *b { "true".to_string() } else { "false".to_string() },
            Expr::Str(s, _) => format!("\"{}\"", escape_c_string(s)),
            Expr::Char(c, _) => format!("((char)'{}')", escape_c_char(*c)),
            Expr::U8(b, _) => format!("((uint8_t){})", b),
            Expr::Ident(name, _) => {
                if name == "None" {
                    "_ox_none()".to_string()
                } else {
                    name.clone()
                }
            }
            Expr::Path { base, member, .. } => {
                if (base == "Option" && member == "None") || (base == "Result" && member == "None") {
                    "_ox_none()".to_string()
                } else {
                    format!("{}_{}", base, member)
                }
            }
            Expr::Unary { op, expr, .. } => {
                let inner_ty = self.infer_expr_type(expr);
                if let Type::Named(ref s_name) = inner_ty {
                    let m_name = match op {
                        UnOp::Neg => "neg",
                        UnOp::Not => "not",
                    };
                    let key = format!("{}_{}", s_name, m_name);
                    if self.fn_ret_types.contains_key(&key) {
                        let inner = self.compile_expr(expr);
                        return if inner == "self" {
                            format!("{}_{}(self)", s_name, m_name)
                        } else if matches!(**expr, Expr::Ident(..)) {
                            format!("{}_{}(&({}))", s_name, m_name, inner)
                        } else {
                            format!("({{ {} _tmp = {}; {}_{}(&_tmp); }})", s_name, inner, s_name, m_name)
                        };
                    }
                }
                let inner = self.compile_expr(expr);
                match op {
                    UnOp::Neg => format!("(-{})", inner),
                    UnOp::Not => format!("(!{})", inner),
                }
            }
            Expr::Binary { op, left, right, .. } => {
                let left_ty = self.infer_expr_type(left);
                if let Type::Named(ref s_name) = left_ty {
                    let method = match op {
                        BinOp::Add => Some("add"),
                        BinOp::Sub => Some("sub"),
                        BinOp::Mul => Some("mul"),
                        BinOp::Div => Some("div"),
                        BinOp::Mod => Some("rem"),
                        BinOp::Eq => Some("eq"),
                        BinOp::NotEq => Some("ne"),
                        BinOp::Lt => Some("lt"),
                        BinOp::LtEq => Some("le"),
                        BinOp::Gt => Some("gt"),
                        BinOp::GtEq => Some("ge"),
                        _ => None,
                    };
                    if let Some(m_name) = method {
                        let key = format!("{}_{}", s_name, m_name);
                        if self.fn_ret_types.contains_key(&key) {
                            let l = self.compile_expr(left);
                            let r = self.compile_expr(right);
                            return if l == "self" {
                                format!("{}_{}(self, {})", s_name, m_name, r)
                            } else if matches!(**left, Expr::Ident(..)) {
                                format!("{}_{}(&({}), {})", s_name, m_name, l, r)
                            } else {
                                format!("({{ {} _tmp = {}; {}_{}(&_tmp, {}); }})", s_name, l, s_name, m_name, r)
                            };
                        } else if *op == BinOp::NotEq {
                            let eq_key = format!("{}_eq", s_name);
                            if self.fn_ret_types.contains_key(&eq_key) {
                                let l = self.compile_expr(left);
                                let r = self.compile_expr(right);
                                return if l == "self" {
                                    format!("((bool)(!{}_eq(self, {})))", s_name, r)
                                } else if matches!(**left, Expr::Ident(..)) {
                                    format!("((bool)(!{}_eq(&({}), {})))", s_name, l, r)
                                } else {
                                    format!("({{ {} _tmp = {}; ((bool)(!{}_eq(&_tmp, {}))); }})", s_name, l, s_name, r)
                                };
                            }
                        }
                    }
                }
                let l = self.compile_expr(left);
                let r = self.compile_expr(right);
                match op {
                    BinOp::Add => {
                        if expr_has_str(left) || expr_has_str(right) {
                            format!("_ox_add_str({}, {})", l, r)
                        } else {
                            format!("({} + {})", l, r)
                        }
                    }
                    BinOp::Sub => format!("({} - {})", l, r),
                    BinOp::Mul => format!("({} * {})", l, r),
                    BinOp::Div => format!("({} / {})", l, r),
                    BinOp::Mod => format!("({} % {})", l, r),
                    BinOp::Eq => format!("({} == {})", l, r),
                    BinOp::NotEq => format!("({} != {})", l, r),
                    BinOp::Lt => format!("({} < {})", l, r),
                    BinOp::LtEq => format!("({} <= {})", l, r),
                    BinOp::Gt => format!("({} > {})", l, r),
                    BinOp::GtEq => format!("({} >= {})", l, r),
                    BinOp::And => format!("({} && {})", l, r),
                    BinOp::Or => format!("({} || {})", l, r),
                }
            }
            Expr::Assign { target, value, .. } => {
                let val = self.compile_expr(value);
                format!("({} = {})", target, val)
            }
            Expr::Array(elements, ..) => {
                let count = elements.len();
                let elem_ty = if let Some(first) = elements.first() {
                    self.infer_expr_type(first)
                } else {
                    Type::Int
                };
                let c_type = self.to_c_type(&elem_ty);
                let mut inits = Vec::new();
                inits.push(format!("{}* _a = ({}*)_ox_array_create_sized({}, sizeof({}));", c_type, c_type, count, c_type));
                for (idx, e) in elements.iter().enumerate() {
                    let val = self.compile_expr(e);
                    inits.push(format!("_a[{}] = {};", idx, val));
                }
                inits.push("_a;".to_string());
                format!("({{ {} }})", inits.join(" "))
            }
            Expr::Index { target, index, .. } => {
                let target_ty = self.infer_expr_type(target);
                let t = self.compile_expr(target);
                let i = self.compile_expr(index);
                match target_ty {
                    Type::Slice(..) => format!("({}.ptr[{}])", t, i),
                    Type::StrSlice => format!("((char){}.ptr[{}])", t, i),
                    Type::Named(ref s_name) => {
                        let key = format!("{}_index", s_name);
                        if self.fn_ret_types.contains_key(&key) {
                            if t == "self" {
                                format!("{}_index(self, {})", s_name, i)
                            } else if matches!(**target, Expr::Ident(..)) {
                                format!("{}_index(&({}), {})", s_name, t, i)
                            } else {
                                format!("({{ {} _tmp = {}; {}_index(&_tmp, {}); }})", s_name, t, s_name, i)
                            }
                        } else {
                            format!("{}[{}]", t, i)
                        }
                    }
                    _ => format!("{}[{}]", t, i),
                }
            }
            Expr::Slice { target, start, end, .. } => {
                let target_ty = self.infer_expr_type(target);
                let t = self.compile_expr(target);
                let s_c = match start {
                    Some(s) => self.compile_expr(s),
                    None => "0LL".to_string(),
                };
                let e_c = match end {
                    Some(e) => self.compile_expr(e),
                    None => format!("len({})", t),
                };
                match target_ty {
                    Type::StrSlice => format!("_ox_str_slice_sub({}, {}, {})", t, s_c, e_c),
                    Type::Str => format!("_ox_str_slice_create({}, {}, {})", t, s_c, e_c),
                    Type::Slice(..) => format!("_ox_slice_sub({}, {}, {})", t, s_c, e_c),
                    _ => format!("_ox_slice_create({}, {}, {})", t, s_c, e_c),
                }
            }
            Expr::IndexAssign { target, index, value, .. } => {
                let target_ty = self.infer_expr_type(target);
                let t = self.compile_expr(target);
                let i = self.compile_expr(index);
                let v = self.compile_expr(value);
                match target_ty {
                    Type::Named(ref s_name) => {
                        let key = format!("{}_index_set", s_name);
                        if self.fn_ret_types.contains_key(&key) {
                            if t == "self" {
                                format!("{}_index_set(self, {}, {})", s_name, i, v)
                            } else if matches!(**target, Expr::Ident(..)) {
                                format!("{}_index_set(&({}), {}, {})", s_name, t, i, v)
                            } else {
                                format!("({{ {} _tmp = {}; {}_index_set(&_tmp, {}, {}); }})", s_name, t, s_name, i, v)
                            }
                        } else {
                            format!("({}[{}] = {})", t, i, v)
                        }
                    }
                    _ => format!("({}[{}] = {})", t, i, v),
                }
            }
            Expr::Tuple(elements, ..) => {
                let elem_tys: Vec<Type> = elements.iter().map(|e| self.infer_expr_type(e)).collect();
                let tuple_name = self.c_tuple_name(&elem_tys);
                let mut field_inits = Vec::new();
                for (idx, e) in elements.iter().enumerate() {
                    field_inits.push(format!("._{} = {}", idx, self.compile_expr(e)));
                }
                format!("(({}){{ {} }})", tuple_name, field_inits.join(", "))
            }
            Expr::TupleIndex { target, index, .. } => {
                let t = self.compile_expr(target);
                format!("(({})._{})", t, index)
            }
            Expr::TupleIndexAssign { target, index, value, .. } => {
                let t = self.compile_expr(target);
                let v = self.compile_expr(value);
                format!("(({})._{} = {})", t, index, v)
            }
            Expr::StructInit { name, fields, .. } => {
                let mut field_inits = Vec::new();
                for (f_name, f_val) in fields {
                    field_inits.push(format!(".{} = {}", f_name, self.compile_expr(f_val)));
                }
                format!("({}){{ {} }}", name, field_inits.join(", "))
            }
            Expr::FieldAccess { target, field, .. } => {
                let target_ty = self.infer_expr_type(target);
                let t = self.compile_expr(target);
                if t == "self" || matches!(target_ty, Type::Box(_) | Type::Rc(_) | Type::RefCell(_) | Type::Ref { .. } | Type::RawPtr { .. }) {
                    format!("{}->{}", t, field)
                } else {
                    format!("{}.{}", t, field)
                }
            }
            Expr::FieldAssign { target, field, value, .. } => {
                let target_ty = self.infer_expr_type(target);
                let t = self.compile_expr(target);
                let v = self.compile_expr(value);
                if t == "self" || matches!(target_ty, Type::Box(_) | Type::Rc(_) | Type::RefCell(_) | Type::Ref { .. } | Type::RawPtr { .. }) {
                    format!("({}->{} = {})", t, field, v)
                } else {
                    format!("({}.{} = {})", t, field, v)
                }
            }
            Expr::MethodCall { target, method, args, .. } => {
                let t = self.compile_expr(target);
                let target_ty = self.infer_expr_type(target);
                match method.as_str() {
                    "is_some" => format!("((bool)({}).is_some)", t),
                    "is_none" => format!("((bool)!({}).is_some)", t),
                    "is_ok" => format!("((bool)({}).is_ok)", t),
                    "is_err" => format!("((bool)!({}).is_ok)", t),
                    "clone" if matches!(target_ty, Type::Rc(_)) => format!("_ox_rc_clone({})", t),
                    "borrow" | "borrow_mut" if matches!(target_ty, Type::RefCell(_)) => t,
                    "unwrap" => {
                        let inner_ty = match target_ty {
                            Type::Option(ref in_t) => Some((**in_t).clone()),
                            Type::Result(ref ok_t, _) => Some((**ok_t).clone()),
                            _ => None,
                        };
                        if let Some(in_t) = inner_ty {
                            if matches!(in_t, Type::Box(_) | Type::Rc(_) | Type::RefCell(_) | Type::Str) {
                                let c_ty = self.to_c_type(&in_t);
                                format!("(({})(uintptr_t)_ox_unwrap({}))", c_ty, t)
                            } else {
                                format!("_ox_unwrap({})", t)
                            }
                        } else {
                            format!("_ox_unwrap({})", t)
                        }
                    }
                    "unwrap_or" => format!("_ox_unwrap_or({}, {})", t, self.compile_expr(&args[0])),
                    "trim" => format!("_ox_str_trim(_ox_to_str({}))", t),
                    "to_upper" | "to_uppercase" => format!("_ox_str_to_upper(_ox_to_str({}))", t),
                    "to_lower" | "to_lowercase" => format!("_ox_str_to_lower(_ox_to_str({}))", t),
                    "contains" => format!("_ox_str_contains(_ox_to_str({}), _ox_to_str({}))", t, self.compile_expr(&args[0])),
                    "split" => format!("_ox_str_split(_ox_to_str({}), _ox_to_str({}))", t, self.compile_expr(&args[0])),
                    "replace" => format!("_ox_str_replace(_ox_to_str({}), _ox_to_str({}), _ox_to_str({}))", t, self.compile_expr(&args[0]), self.compile_expr(&args[1])),
                    "char_at" => format!("_ox_char_at(_ox_to_str({}), {})", t, self.compile_expr(&args[0])),
                    "push" => format!("push({}, {})", t, self.compile_expr(&args[0])),
                    "pop" => format!("pop({})", t),
                    "len" => format!("len({})", t),
                    "map" => format!("_ox_arr_map({}, {})", t, self.compile_expr(&args[0])),
                    "filter" => format!("_ox_arr_filter({}, {})", t, self.compile_expr(&args[0])),
                    "set" => format!("_ox_map_set({}, {}, _ox_to_str({}))", t, self.compile_expr(&args[0]), self.compile_expr(&args[1])),
                    "get" => format!("_ox_map_get({}, {})", t, self.compile_expr(&args[0])),
                    "has" => format!("_ox_map_has({}, {})", t, self.compile_expr(&args[0])),
                    "keys" => format!("_ox_map_keys({})", t),
                    "to_array" | "to_vec" => {
                        format!("({{ _OxSlice _s = ({}); int64_t* _a = _ox_array_create(_s.len); if (_s.ptr && _s.len > 0) memcpy(_a, _s.ptr, _s.len * sizeof(int64_t)); _a; }})", t)
                    }
                    "to_str" | "to_string" => format!("_ox_to_str_str_slice({})", t),
                    _ => {
                        let mut arg_strs = Vec::new();
                        if t == "self" || matches!(target_ty, Type::Box(_) | Type::Rc(_) | Type::RefCell(_) | Type::Ref { .. } | Type::RawPtr { .. }) {
                            arg_strs.push(t);
                        } else {
                            arg_strs.push(format!("&({})", t));
                        }
                        for a in args {
                            arg_strs.push(self.compile_expr(a));
                        }
                        let mut curr_ty = target_ty;
                        while let Type::Box(in_t) | Type::Rc(in_t) | Type::RefCell(in_t) = curr_ty {
                            curr_ty = *in_t;
                        }
                        let struct_name = if let Expr::Ident(ref name, _) = **target {
                            if name == "self" {
                                self.current_impl.clone().unwrap_or_else(|| "Unknown".to_string())
                            } else if let Some(Type::Named(s)) = self.var_ox_types.get(name) {
                                s.clone()
                            } else if let Some(t) = self.var_types.get(name) {
                                t.clone()
                            } else {
                                match curr_ty {
                                    Type::Named(s) => s,
                                    _ => self.method_to_struct.get(method).cloned().unwrap_or_else(|| "Unknown".to_string()),
                                }
                            }
                        } else {
                            match curr_ty {
                                Type::Named(s) => s,
                                _ => self.method_to_struct.get(method).cloned().unwrap_or_else(|| "Unknown".to_string()),
                            }
                        };

                        format!("{}_{}({})", struct_name, method, arg_strs.join(", "))
                    }
                }
            }
            Expr::Call { callee, args, .. } => {
                match callee.as_str() {
                    "assert" => {
                        let cond = self.compile_expr(&args[0]);
                        format!("assert({})", cond)
                    }
                    "println" => {
                        let mut prints = Vec::new();
                        for (idx, a) in args.iter().enumerate() {
                            let arg_s = self.compile_expr(a);
                            if idx > 0 {
                                prints.push("printf(\" \");".to_string());
                            }
                            prints.push(format!("_ox_print_item({});", arg_s));
                        }
                        prints.push("printf(\"\\n\");".to_string());
                        format!("({{ {}; }})", prints.join(" "))
                    }
                    "print" => {
                        let mut prints = Vec::new();
                        for (idx, a) in args.iter().enumerate() {
                            let arg_s = self.compile_expr(a);
                            if idx > 0 {
                                prints.push("printf(\" \");".to_string());
                            }
                            prints.push(format!("_ox_print_item({});", arg_s));
                        }
                        format!("({{ {}; }})", prints.join(" "))
                    }
                    "trim" => format!("_ox_str_trim(_ox_to_str({}))", self.compile_expr(&args[0])),
                    "to_upper" | "to_uppercase" => format!("_ox_str_to_upper(_ox_to_str({}))", self.compile_expr(&args[0])),
                    "to_lower" | "to_lowercase" => format!("_ox_str_to_lower(_ox_to_str({}))", self.compile_expr(&args[0])),
                    "contains" => format!("_ox_str_contains(_ox_to_str({}), _ox_to_str({}))", self.compile_expr(&args[0]), self.compile_expr(&args[1])),
                    "split" => format!("_ox_str_split(_ox_to_str({}), _ox_to_str({}))", self.compile_expr(&args[0]), self.compile_expr(&args[1])),
                    "replace" => format!("_ox_str_replace(_ox_to_str({}), _ox_to_str({}), _ox_to_str({}))", self.compile_expr(&args[0]), self.compile_expr(&args[1]), self.compile_expr(&args[2])),
                    "char_at" => format!("_ox_char_at(_ox_to_str({}), {})", self.compile_expr(&args[0]), self.compile_expr(&args[1])),
                    "args" => "_ox_args()".to_string(),
                    "get_env" => format!("_ox_get_env({})", self.compile_expr(&args[0])),
                    "exit" => format!("exit((int)({}))", self.compile_expr(&args[0])),
                    "map_new" => "_ox_map_new()".to_string(),
                    "map_set" => format!("_ox_map_set({}, {}, _ox_to_str({}))", self.compile_expr(&args[0]), self.compile_expr(&args[1]), self.compile_expr(&args[2])),
                    "map_get" => format!("_ox_map_get({}, {})", self.compile_expr(&args[0]), self.compile_expr(&args[1])),
                    "map_has" => format!("_ox_map_has({}, {})", self.compile_expr(&args[0]), self.compile_expr(&args[1])),
                    "map_len" => format!("_ox_map_len({})", self.compile_expr(&args[0])),
                    "map_keys" => format!("_ox_map_keys({})", self.compile_expr(&args[0])),
                    "arena_reset" => "({ _ox_arena_pos = 0; 0; })".to_string(),
                    "Box::new" => {
                        let val_code = self.compile_expr(&args[0]);
                        let val_ty = self.infer_expr_type(&args[0]);
                        let c_ty = self.to_c_type(&val_ty);
                        format!("({{\n    {}* _b = ({}*)_ox_arena_alloc(sizeof({}));\n    *_b = {};\n    _b;\n}})", c_ty, c_ty, c_ty, val_code)
                    }
                    "Rc::new" => {
                        let val_code = self.compile_expr(&args[0]);
                        let val_ty = self.infer_expr_type(&args[0]);
                        let c_ty = self.to_c_type(&val_ty);
                        format!("({{\n    _OxRc* _r = (_OxRc*)_ox_arena_alloc(sizeof(_OxRc));\n    _r->count = 1;\n    {}* _val_ptr = ({}*)_ox_arena_alloc(sizeof({}));\n    *_val_ptr = {};\n    _r->val = (void*)_val_ptr;\n    _r;\n}})", c_ty, c_ty, c_ty, val_code)
                    }
                    "Rc::strong_count" => {
                        let r_code = self.compile_expr(&args[0]);
                        format!("(({})->count)", r_code)
                    }
                    "RefCell::new" => {
                        let val_code = self.compile_expr(&args[0]);
                        let val_ty = self.infer_expr_type(&args[0]);
                        let c_ty = self.to_c_type(&val_ty);
                        format!("({{\n    {}* _b = ({}*)_ox_arena_alloc(sizeof({}));\n    *_b = {};\n    _b;\n}})", c_ty, c_ty, c_ty, val_code)
                    }
                    "Some" | "Option_Some" => format!("_ox_some((int64_t)(uintptr_t)({}))", self.compile_expr(&args[0])),
                    "Ok" | "Result_Ok" => format!("_ox_ok((int64_t)({}))", self.compile_expr(&args[0])),
                    "Err" | "Result_Err" => format!("_ox_err((int64_t)({}))", self.compile_expr(&args[0])),
                    _ => {
                        let mut arg_strs = Vec::new();
                        let expected_param_types = self.fn_param_types.get(callee).cloned();
                        for (idx, a) in args.iter().enumerate() {
                            let arg_s = self.compile_expr(a);
                            let arg_ty = self.infer_expr_type(a);
                            if let Some(ref params) = expected_param_types {
                                if let Some(param_ty) = params.get(idx) {
                                    if (matches!(param_ty, Type::Slice(..)) || matches!(param_ty, Type::Ref { inner, .. } if matches!(**inner, Type::Array(..))))
                                        && matches!(arg_ty, Type::Array(..))
                                    {
                                        arg_strs.push(format!("_ox_slice_create({}, 0LL, len({}))", arg_s, arg_s));
                                        continue;
                                    }
                                    if (*param_ty == Type::StrSlice || matches!(param_ty, Type::Ref { inner, .. } if **inner == Type::Str))
                                        && arg_ty == Type::Str
                                    {
                                        arg_strs.push(format!("_ox_str_slice_create({}, 0LL, len({}))", arg_s, arg_s));
                                        continue;
                                    }
                                }
                            }
                            arg_strs.push(arg_s);
                        }
                        format!("{}({})", callee, arg_strs.join(", "))
                    }
                }
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                let cond_str = self.compile_expr(condition);
                if then_branch.stmts.is_empty() && else_branch.as_ref().map_or(true, |eb| eb.stmts.is_empty()) {
                    let then_val = then_branch.trailing_expr.as_ref().map(|t| self.compile_expr(t)).unwrap_or_else(|| "0".to_string());
                    let else_val = else_branch.as_ref().and_then(|e| e.trailing_expr.as_ref()).map(|t| self.compile_expr(t)).unwrap_or_else(|| "0".to_string());
                    format!("(({}) ? ({}) : ({}))", cond_str, then_val, else_val)
                } else {
                    let mut then_code = Vec::new();
                    for s in &then_branch.stmts {
                        match s {
                            Stmt::Expr(e) => then_code.push(format!("{};", self.compile_expr(e))),
                            Stmt::Let { name, ty, init, .. } => {
                                let type_str = if let Some(t) = ty { self.to_c_type(t) } else { "__auto_type".to_string() };
                                if let Some(i) = init {
                                    then_code.push(format!("{} {} = {};", type_str, name, self.compile_expr(i)));
                                } else {
                                    then_code.push(format!("{} {};", type_str, name));
                                }
                            }
                            _ => {}
                        }
                    }
                    let then_trail = then_branch.trailing_expr.as_ref().map(|t| format!("{};", self.compile_expr(t))).unwrap_or_default();

                    let mut else_code = Vec::new();
                    if let Some(eb) = else_branch {
                        for s in &eb.stmts {
                            match s {
                                Stmt::Expr(e) => else_code.push(format!("{};", self.compile_expr(e))),
                                Stmt::Let { name, ty, init, .. } => {
                                    let type_str = if let Some(t) = ty { self.to_c_type(t) } else { "__auto_type".to_string() };
                                    if let Some(i) = init {
                                        else_code.push(format!("{} {} = {};", type_str, name, self.compile_expr(i)));
                                    } else {
                                        else_code.push(format!("{} {};", type_str, name));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    let else_trail = else_branch.as_ref().and_then(|e| e.trailing_expr.as_ref()).map(|t| format!("{};", self.compile_expr(t))).unwrap_or_default();

                    format!("({{ if ({}) {{ {} {} }} else {{ {} {} }} 0; }})",
                        cond_str,
                        then_code.join(" "), then_trail,
                        else_code.join(" "), else_trail
                    )
                }
            }
            Expr::Block(block) => {
                let mut stmts_str = Vec::new();
                for s in &block.stmts {
                    match s {
                        Stmt::Expr(e) => stmts_str.push(format!("{};", self.compile_expr(e))),
                        _ => {}
                    }
                }
                let trail = block.trailing_expr.as_ref().map(|t| self.compile_expr(t)).unwrap_or_else(|| "0".to_string());
                format!("({{ {} {}; }})", stmts_str.join(" "), trail)
            }
            Expr::UnsafeBlock(block, _) => {
                self.compile_expr(&Expr::Block(block.clone()))
            }
            Expr::Borrow { expr, .. } => {
                format!("(&({}))", self.compile_expr(expr))
            }
            Expr::Deref { expr, .. } => {
                let inner_code = self.compile_expr(expr);
                let inner_ty = self.infer_expr_type(expr);
                if let Type::Rc(ref t) = inner_ty {
                    let c_ty = self.to_c_type(t);
                    format!("(*(({}*)(({})->val)))", c_ty, inner_code)
                } else {
                    format!("(*({}))", inner_code)
                }
            }
            Expr::DerefAssign { target, value, .. } => {
                let tgt_code = self.compile_expr(target);
                let val_code = self.compile_expr(value);
                let tgt_ty = self.infer_expr_type(target);
                if let Type::Rc(ref t) = tgt_ty {
                    let c_ty = self.to_c_type(t);
                    format!("(*(({}*)(({})->val)) = {})", c_ty, tgt_code, val_code)
                } else {
                    format!("(*({}) = {})", tgt_code, val_code)
                }
            }
            Expr::Closure { params, body, .. } => {
                let param_strs = params
                    .iter()
                    .map(|p| format!("int64_t {}", p.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut stmts_str = Vec::new();
                for s in &body.stmts {
                    match s {
                        Stmt::Expr(e) => stmts_str.push(format!("{};", self.compile_expr(e))),
                        Stmt::Let { name, ty, init, .. } => {
                            let type_str = if let Some(t) = ty { self.to_c_type(t) } else { "__auto_type".to_string() };
                            if let Some(i) = init {
                                stmts_str.push(format!("{} {} = {};", type_str, name, self.compile_expr(i)));
                            } else {
                                stmts_str.push(format!("{} {};", type_str, name));
                            }
                        }
                        _ => {}
                    }
                }
                let trail = body.trailing_expr.as_ref().map(|t| self.compile_expr(t)).unwrap_or_else(|| "0".to_string());
                format!("({{ int64_t _closure({}) {{ {} return {}; }} _closure; }})", param_strs, stmts_str.join(" "), trail)
            }
            Expr::Match { subject, arms, .. } => {
                let subj = self.compile_expr(subject);
                let inferred_subj = self.infer_expr_type(subject);
                let subj_ty: Option<Type> = if inferred_subj != Type::Void {
                    Some(inferred_subj)
                } else if let Expr::Ident(name, _) = subject.as_ref() {
                    self.var_ox_types.get(name).cloned()
                } else {
                    None
                };
                let mut expr = "0".to_string();

                for arm in arms.iter().rev() {
                    let body_str = self.compile_expr(&arm.body);
                    match &arm.pattern {
                        Pattern::Wildcard(_) => {
                            expr = body_str;
                        }
                        Pattern::Int(i, _) => {
                            expr = format!("((_subj == {}LL) ? ({}) : ({}))", i, body_str, expr);
                        }
                        Pattern::Float(fl, _) => {
                            expr = format!("((_subj == {}) ? ({}) : ({}))", fl, body_str, expr);
                        }
                        Pattern::Bool(b, _) => {
                            expr = format!("((_subj == {}) ? ({}) : ({}))", b, body_str, expr);
                        }
                        Pattern::Str(s, _) => {
                            expr = format!("((strcmp(_subj, \"{}\") == 0) ? ({}) : ({}))", escape_c_string(s), body_str, expr);
                        }
                        Pattern::Char(c, _) => {
                            expr = format!("((_subj == '{}') ? ({}) : ({}))", escape_c_char(*c), body_str, expr);
                        }
                        Pattern::U8(b, _) => {
                            expr = format!("((_subj == ((uint8_t){})) ? ({}) : ({}))", b, body_str, expr);
                        }
                        Pattern::EnumVariant { enum_name, variant, .. } => {
                            expr = format!("((_subj == {}_{}) ? ({}) : ({}))", enum_name, variant, body_str, expr);
                        }
                        Pattern::Ident(name, _) => {
                            expr = format!("({{ __auto_type {} = _subj; {}; }})", name, body_str);
                        }
                        Pattern::Some(inner, _) => {
                            let cast_type = match &subj_ty {
                                Some(Type::Option(in_t)) => self.to_c_type(in_t),
                                _ => "__auto_type".to_string(),
                            };
                            match &**inner {
                                Pattern::Ident(name, _) => {
                                    if cast_type == "__auto_type" {
                                        expr = format!("((_subj.is_some) ? ({{ __auto_type {} = _subj.val; {}; }}) : ({}))", name, body_str, expr);
                                    } else {
                                        expr = format!("((_subj.is_some) ? ({{ {} {} = ({})(uintptr_t)_subj.val; {}; }}) : ({}))", cast_type, name, cast_type, body_str, expr);
                                    }
                                }
                                Pattern::Wildcard(_) => {
                                    expr = format!("((_subj.is_some) ? ({}) : ({}))", body_str, expr);
                                }
                                Pattern::Int(i, _) => {
                                    expr = format!("((_subj.is_some && _subj.val == {}LL) ? ({}) : ({}))", i, body_str, expr);
                                }
                                Pattern::Bool(b, _) => {
                                    expr = format!("((_subj.is_some && _subj.val == {}) ? ({}) : ({}))", b, body_str, expr);
                                }
                                Pattern::Str(s, _) => {
                                    expr = format!("((_subj.is_some && strcmp((const char*)_subj.val, \"{}\") == 0) ? ({}) : ({}))", escape_c_string(s), body_str, expr);
                                }
                                _ => {
                                    expr = format!("((_subj.is_some) ? ({}) : ({}))", body_str, expr);
                                }
                            }
                        }
                        Pattern::None(_) => {
                            expr = format!("((!_subj.is_some) ? ({}) : ({}))", body_str, expr);
                        }
                        Pattern::Ok(inner, _) => {
                            let cast_type = match &subj_ty {
                                Some(Type::Result(ok_t, _)) => self.to_c_type(ok_t),
                                _ => "__auto_type".to_string(),
                            };
                            match &**inner {
                                Pattern::Ident(name, _) => {
                                    if cast_type == "__auto_type" {
                                        expr = format!("((_subj.is_ok) ? ({{ __auto_type {} = _subj.ok; {}; }}) : ({}))", name, body_str, expr);
                                    } else {
                                        expr = format!("((_subj.is_ok) ? ({{ {} {} = ({})_subj.ok; {}; }}) : ({}))", cast_type, name, cast_type, body_str, expr);
                                    }
                                }
                                Pattern::Wildcard(_) => {
                                    expr = format!("((_subj.is_ok) ? ({}) : ({}))", body_str, expr);
                                }
                                Pattern::Int(i, _) => {
                                    expr = format!("((_subj.is_ok && _subj.ok == {}LL) ? ({}) : ({}))", i, body_str, expr);
                                }
                                Pattern::Bool(b, _) => {
                                    expr = format!("((_subj.is_ok && _subj.ok == {}) ? ({}) : ({}))", b, body_str, expr);
                                }
                                Pattern::Str(s, _) => {
                                    expr = format!("((_subj.is_ok && strcmp((const char*)_subj.ok, \"{}\") == 0) ? ({}) : ({}))", escape_c_string(s), body_str, expr);
                                }
                                _ => {
                                    expr = format!("((_subj.is_ok) ? ({}) : ({}))", body_str, expr);
                                }
                            }
                        }
                        Pattern::Err(inner, _) => {
                            let cast_type = match &subj_ty {
                                Some(Type::Result(_, err_t)) => self.to_c_type(err_t),
                                _ => "__auto_type".to_string(),
                            };
                            match &**inner {
                                Pattern::Ident(name, _) => {
                                    if cast_type == "__auto_type" {
                                        expr = format!("((!_subj.is_ok) ? ({{ __auto_type {} = _subj.err; {}; }}) : ({}))", name, body_str, expr);
                                    } else {
                                        expr = format!("((!_subj.is_ok) ? ({{ {} {} = ({})_subj.err; {}; }}) : ({}))", cast_type, name, cast_type, body_str, expr);
                                    }
                                }
                                Pattern::Wildcard(_) => {
                                    expr = format!("((!_subj.is_ok) ? ({}) : ({}))", body_str, expr);
                                }
                                Pattern::Str(s, _) => {
                                    expr = format!("((!_subj.is_ok && strcmp((const char*)_subj.err, \"{}\") == 0) ? ({}) : ({}))", escape_c_string(s), body_str, expr);
                                }
                                _ => {
                                    expr = format!("((!_subj.is_ok) ? ({}) : ({}))", body_str, expr);
                                }
                            }
                        }
                        Pattern::Tuple(pats, _) => {
                            let mut conds = Vec::new();
                            let mut bindings = Vec::new();
                            for (idx, p) in pats.iter().enumerate() {
                                match p {
                                    Pattern::Wildcard(_) => {}
                                    Pattern::Ident(name, _) => {
                                        if name != "_" {
                                            bindings.push(format!("__auto_type {} = _subj._{};", name, idx));
                                        }
                                    }
                                    Pattern::Int(i, _) => {
                                        conds.push(format!("(_subj._{} == {}LL)", idx, i));
                                    }
                                    Pattern::Float(fl, _) => {
                                        conds.push(format!("(_subj._{} == {})", idx, fl));
                                    }
                                    Pattern::Bool(b, _) => {
                                        conds.push(format!("(_subj._{} == {})", idx, b));
                                    }
                                    Pattern::Str(s, _) => {
                                        conds.push(format!("(strcmp(_subj._{}, \"{}\") == 0)", idx, escape_c_string(s)));
                                    }
                                    Pattern::Char(c, _) => {
                                        conds.push(format!("(_subj._{} == '{}')", idx, escape_c_char(*c)));
                                    }
                                    Pattern::U8(b, _) => {
                                        conds.push(format!("(_subj._{} == ((uint8_t){}))", idx, b));
                                    }
                                    _ => {}
                                }
                            }
                            let cond_str = if conds.is_empty() {
                                "1".to_string()
                            } else {
                                conds.join(" && ")
                            };
                            let bind_str = if bindings.is_empty() {
                                String::new()
                            } else {
                                format!("{} ", bindings.join(" "))
                            };
                            expr = format!("(({}) ? ({{ {}{}; }}) : ({}))", cond_str, bind_str, body_str, expr);
                        }
                    }
                }

                format!("({{ __auto_type _subj = {}; {}; }})", subj, expr)
            }
            Expr::Try(inner, _) => {
                let inner_c = self.compile_expr(inner);
                if let Some(Type::Result(..)) = &self.current_fn_ret {
                    format!("({{ __auto_type _try_val = ({}); if (!_try_val.is_ok) return (_OxResult){{ .is_ok = false, .ok = 0, .err = _try_val.err }}; _try_val.ok; }})", inner_c)
                } else {
                    format!("({{ __auto_type _try_val = ({}); if (!_try_val.is_some) return _ox_none(); _try_val.val; }})", inner_c)
                }
            }
            Expr::FormatString(parts, _) => {
                if parts.is_empty() {
                    "\"\"".to_string()
                } else {
                    let mut cur = match &parts[0] {
                        FormatPart::Lit(s) => format!("\"{}\"", escape_c_string(s)),
                        FormatPart::Expr(e) => format!("_ox_to_str({})", self.compile_expr(e)),
                    };
                    for p in &parts[1..] {
                        let next_str = match p {
                            FormatPart::Lit(s) => format!("\"{}\"", escape_c_string(s)),
                            FormatPart::Expr(e) => format!("_ox_to_str({})", self.compile_expr(e)),
                        };
                        cur = format!("_ox_str_concat({}, {})", cur, next_str);
                    }
                    cur
                }
            }
        }
    }
}
