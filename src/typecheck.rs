use std::collections::{HashMap, HashSet};
use crate::ast::*;
use crate::span::Span;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct VarSymbol {
    ty: Type,
    is_mut: bool,
    span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Char(char),
    U8(u8),
}

impl ConstValue {
    pub fn get_type(&self) -> Type {
        match self {
            ConstValue::Int(_) => Type::Int,
            ConstValue::Float(_) => Type::Float,
            ConstValue::Bool(_) => Type::Bool,
            ConstValue::Str(_) => Type::Str,
            ConstValue::Char(_) => Type::Char,
            ConstValue::U8(_) => Type::U8,
        }
    }
}

pub struct TypeChecker {
    scopes: Vec<HashMap<String, VarSymbol>>,
    constants: HashMap<String, (Type, ConstValue)>,
    functions: HashMap<String, Function>,
    extern_functions: HashSet<String>,
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, EnumDef>,
    traits: HashMap<String, TraitDef>,
    trait_impls: HashMap<String, HashSet<String>>,
    methods: HashMap<String, HashMap<String, Function>>, // struct_name -> (method_name -> Function)
    current_fn_return: Option<Type>,
    loop_depth: usize,
    is_unsafe: bool,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            constants: HashMap::new(),
            functions: HashMap::new(),
            extern_functions: HashSet::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            traits: HashMap::new(),
            trait_impls: HashMap::new(),
            methods: HashMap::new(),
            current_fn_return: None,
            loop_depth: 0,
            is_unsafe: false,
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_var(&mut self, name: String, ty: Type, is_mut: bool, span: Span) -> Result<(), String> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, VarSymbol { ty, is_mut, span });
            Ok(())
        } else {
            Err("İç hata: Kapsam bulunamadı".to_string())
        }
    }

    fn lookup_var(&self, name: &str, span: Span) -> Result<VarSymbol, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Ok(sym.clone());
            }
        }
        Err(format!(
            "Tip Hatası [{}:{}]: Tanımsız değişken '{}'",
            span.line, span.col, name
        ))
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), String> {
        // 1. Pass: Collect all global items (Structs, Enums, Function signatures, Methods)
        for item in &program.items {
            match item {
                Item::Struct(s) => {
                    if self.structs.contains_key(&s.name) {
                        return Err(format!("Tip Hatası [{}:{}]: '{}' struct'ı zaten tanımlanmış", s.span.line, s.span.col, s.name));
                    }
                    self.structs.insert(s.name.clone(), s.clone());
                }
                Item::Enum(e) => {
                    if self.enums.contains_key(&e.name) {
                        return Err(format!("Tip Hatası [{}:{}]: '{}' enum'ı zaten tanımlanmış", e.span.line, e.span.col, e.name));
                    }
                    self.enums.insert(e.name.clone(), e.clone());
                }
                Item::Function(f) => {
                    if self.functions.contains_key(&f.name) {
                        return Err(format!("Tip Hatası [{}:{}]: '{}' fonksiyonu zaten tanımlanmış", f.span.line, f.span.col, f.name));
                    }
                    self.functions.insert(f.name.clone(), f.clone());
                }
                Item::Trait(t) => {
                    if self.traits.contains_key(&t.name) {
                        return Err(format!("Tip Hatası [{}:{}]: '{}' trait'i zaten tanımlanmış", t.span.line, t.span.col, t.name));
                    }
                    self.traits.insert(t.name.clone(), t.clone());
                }
                Item::Const(c) => {
                    if self.constants.contains_key(&c.name) {
                        return Err(format!("Tip Hatası [{}:{}]: '{}' sabiti zaten tanımlanmış", c.span.line, c.span.col, c.name));
                    }
                    let const_val = self.eval_const_expr(&c.value)?;
                    if !self.are_types_compatible(&c.ty, &const_val.get_type()) {
                        return Err(format!(
                            "Tip Hatası [{}:{}]: '{}' sabiti '{}' beklerken '{}' hesaplandı",
                            c.span.line, c.span.col, c.name, c.ty, const_val.get_type()
                        ));
                    }
                    self.constants.insert(c.name.clone(), (c.ty.clone(), const_val));
                    self.declare_var(c.name.clone(), c.ty.clone(), false, c.span)?;
                }
                Item::Impl(imp) => {
                    if let Some(ref tr) = imp.trait_name {
                        self.trait_impls.entry(imp.target.clone()).or_insert_with(HashSet::new).insert(tr.clone());
                    }
                    let entry = self.methods.entry(imp.target.clone()).or_insert_with(HashMap::new);
                    for m in &imp.methods {
                        entry.insert(m.name.clone(), m.clone());
                    }
                }
                Item::Extern(ext) => {
                    for f in &ext.functions {
                        if self.functions.contains_key(&f.name) {
                            return Err(format!("Tip Hatası [{}:{}]: '{}' fonksiyonu zaten tanımlanmış", f.span.line, f.span.col, f.name));
                        }
                        self.extern_functions.insert(f.name.clone());
                        self.functions.insert(f.name.clone(), Function {
                            name: f.name.clone(),
                            generic_params: Vec::new(),
                            params: f.params.clone(),
                            return_type: f.return_type.clone(),
                            body: Block { stmts: Vec::new(), trailing_expr: None, span: f.span },
                            span: f.span,
                        });
                    }
                }
                Item::Import(..) => {}
            }
        }

        // Verify main function exists unless this is a test file
        let has_test_fn = self.functions.keys().any(|k| k.starts_with("test"));
        if !self.functions.contains_key("main") && !has_test_fn {
            return Err("Tip Hatası: Programda 'fn main()' giriş fonksiyonu bulunamadı.".to_string());
        }

        // 2. Pass: Type check function and method bodies
        for item in &program.items {
            match item {
                Item::Function(f) => {
                    self.check_function(f, None)?;
                }
                Item::Impl(imp) => {
                    if !self.structs.contains_key(&imp.target) {
                        return Err(format!("Tip Hatası [{}:{}]: Tanımsız struct '{}' için 'impl' bloğu yazılamaz", imp.span.line, imp.span.col, imp.target));
                    }
                    if let Some(ref tr) = imp.trait_name {
                        let is_standard_trait = matches!(tr.as_str(), "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Neg" | "Not" | "Eq" | "Index");
                        let trait_def = match self.traits.get(tr) {
                            Some(t) => Some(t.clone()),
                            None if is_standard_trait => None,
                            None => return Err(format!("Tip Hatası [{}:{}]: Tanımsız trait '{}'", imp.span.line, imp.span.col, tr)),
                        };

                        if let Some(trait_def) = trait_def {
                            // Check all required trait methods are implemented
                            for tm in &trait_def.methods {
                                if tm.default_body.is_none() && !imp.methods.iter().any(|m| m.name == tm.name) {
                                    return Err(format!(
                                        "Tip Hatası [{}:{}]: '{}' struct'ı '{}' trait'inin zorunlu '{}' metodunu uygulamamış",
                                        imp.span.line, imp.span.col, imp.target, tr, tm.name
                                    ));
                                }
                            }

                            // Check method signatures match
                            for m in &imp.methods {
                                if let Some(tm) = trait_def.methods.iter().find(|t| t.name == m.name) {
                                    if m.params.len() != tm.params.len() {
                                        return Err(format!(
                                            "Tip Hatası [{}:{}]: '{}' metodunun parametre sayısı ({}) '{}' trait'indeki tanım ({}) ile uyuşmuyor",
                                            m.span.line, m.span.col, m.name, m.params.len(), tr, tm.params.len()
                                        ));
                                    }
                                    if m.return_type != tm.return_type && tm.return_type != Type::Void {
                                        return Err(format!(
                                            "Tip Hatası [{}:{}]: '{}' metodunun dönüş tipi ('{}') '{}' trait'indeki dönüş tipi ('{}') ile uyuşmuyor",
                                            m.span.line, m.span.col, m.name, m.return_type, tr, tm.return_type
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    for m in &imp.methods {
                        self.check_function(m, Some(&imp.target))?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn are_types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual || *actual == Type::Void || *expected == Type::Void {
            return true;
        }
        if *expected == Type::U8 && *actual == Type::Int {
            return true;
        }
        if let (Type::Option(d_in), Type::Option(i_in)) = (expected, actual) {
            return **i_in == Type::Void || self.are_types_compatible(d_in, i_in);
        }
        if let (Type::Result(d_ok, d_err), Type::Result(i_ok, i_err)) = (expected, actual) {
            return (**i_ok == Type::Void || self.are_types_compatible(d_ok, i_ok))
                && (**i_err == Type::Void || self.are_types_compatible(d_err, i_err));
        }
        if let (Type::Tuple(t1), Type::Tuple(t2)) = (expected, actual) {
            if t1.len() != t2.len() {
                return false;
            }
            return t1.iter().zip(t2.iter()).all(|(a, b)| self.are_types_compatible(a, b));
        }
        if let (Type::Box(d_in), Type::Box(i_in)) = (expected, actual) {
            return self.are_types_compatible(d_in, i_in);
        }
        if let (Type::Rc(d_in), Type::Rc(i_in)) = (expected, actual) {
            return self.are_types_compatible(d_in, i_in);
        }
        if let (Type::RefCell(d_in), Type::RefCell(i_in)) = (expected, actual) {
            return self.are_types_compatible(d_in, i_in);
        }
        if let (Type::JoinHandle(d_in), Type::JoinHandle(i_in)) = (expected, actual) {
            return self.are_types_compatible(d_in, i_in);
        }
        if let (Type::Sender(d_in), Type::Sender(i_in)) = (expected, actual) {
            return **i_in == Type::Void || self.are_types_compatible(d_in, i_in);
        }
        if let (Type::Receiver(d_in), Type::Receiver(i_in)) = (expected, actual) {
            return **i_in == Type::Void || self.are_types_compatible(d_in, i_in);
        }
        if let (Type::Mutex(d_in), Type::Mutex(i_in)) = (expected, actual) {
            return self.are_types_compatible(d_in, i_in);
        }
        if let (Type::Slice(exp_in), Type::Array(act_in)) = (expected, actual) {
            return self.are_types_compatible(exp_in, act_in);
        }
        if let (Type::Slice(exp_in), Type::Slice(act_in)) = (expected, actual) {
            return self.are_types_compatible(exp_in, act_in);
        }
        if (*expected == Type::StrSlice && *actual == Type::Str)
            || (*expected == Type::Str && *actual == Type::StrSlice)
        {
            return true;
        }
        false
    }

    fn check_function(&mut self, f: &Function, impl_target: Option<&str>) -> Result<(), String> {
        self.enter_scope();
        self.current_fn_return = Some(f.return_type.clone());

        for param in &f.params {
            let ty = if param.name == "self" {
                if let Some(target) = impl_target {
                    Type::Named(target.to_string())
                } else {
                    param.ty.clone()
                }
            } else {
                param.ty.clone()
            };
            self.declare_var(param.name.clone(), ty, param.is_mut, f.span)?;
        }

        let body_type = self.check_block(&f.body)?;

        // Verify return type compatibility
        let is_ret_compatible = self.are_types_compatible(&f.return_type, &body_type);
        if f.return_type != Type::Void && !is_ret_compatible {
            return Err(format!(
                "Tip Hatası [{}:{}]: '{}' fonksiyonu '{}' dönüş tipi beklerken '{}' tipi döndürüldü",
                f.span.line, f.span.col, f.name, f.return_type, body_type
            ));
        }

        self.current_fn_return = None;
        self.exit_scope();
        Ok(())
    }

    fn check_block(&mut self, block: &Block) -> Result<Type, String> {
        self.enter_scope();

        for stmt in &block.stmts {
            self.check_stmt(stmt)?;
        }

        let trailing_ty = if let Some(t) = &block.trailing_expr {
            self.check_expr(t)?
        } else {
            Type::Void
        };

        self.exit_scope();
        Ok(trailing_ty)
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, is_mut, ty, init, span } => {
                let init_ty = if let Some(expr) = init {
                    Some(self.check_expr(expr)?)
                } else {
                    None
                };

                let final_ty = match (ty, init_ty) {
                    (Some(declared), Some(inferred)) => {
                        if !self.are_types_compatible(declared, &inferred) {
                            return Err(format!(
                                "Tip Hatası [{}:{}]: '{}' değişkenine '{}' tipi atanırken, değişken '{}' olarak tanımlanmış",
                                span.line, span.col, name, inferred, declared
                            ));
                        }
                        declared.clone()
                    }
                    (Some(declared), None) => declared.clone(),
                    (None, Some(inferred)) => inferred,
                    (None, None) => {
                        return Err(format!(
                            "Tip Hatası [{}:{}]: '{}' değişkeni için ne tip ne de ilk değer belirtilmiş",
                            span.line, span.col, name
                        ));
                    }
                };

                self.declare_var(name.clone(), final_ty, *is_mut, *span)?;
                Ok(())
            }
            Stmt::LetTuple { elements, ty, init, span } => {
                let init_ty = if let Some(expr) = init {
                    Some(self.check_expr(expr)?)
                } else {
                    None
                };

                let tuple_ty = match (ty, init_ty) {
                    (Some(declared), Some(inferred)) => {
                        if !self.are_types_compatible(declared, &inferred) {
                            return Err(format!(
                                "Tip Hatası [{}:{}]: Demet atamasında beklenen tip '{}', bulunan '{}'",
                                span.line, span.col, declared, inferred
                            ));
                        }
                        declared.clone()
                    }
                    (Some(declared), None) => declared.clone(),
                    (None, Some(inferred)) => inferred,
                    (None, None) => {
                        return Err(format!(
                            "Tip Hatası [{}:{}]: Demet değişkenleri için ne tip ne de ilk değer belirtilmiş",
                            span.line, span.col
                        ));
                    }
                };

                let elem_tys = match tuple_ty {
                    Type::Tuple(tys) => tys,
                    _ => {
                        return Err(format!(
                            "Tip Hatası [{}:{}]: Demet ayrıştırma (destructuring) sadece demet tipleri üzerinde yapılabilir. Bulunan: '{}'",
                            span.line, span.col, tuple_ty
                        ));
                    }
                };

                if elements.len() != elem_tys.len() {
                    return Err(format!(
                        "Tip Hatası [{}:{}]: Demet ayrıştırma eleman sayısı uyuşmuyor: {} değişken beklenirken demette {} eleman var",
                        span.line, span.col, elements.len(), elem_tys.len()
                    ));
                }

                for ((name, is_mut), elem_ty) in elements.iter().zip(elem_tys.into_iter()) {
                    if name != "_" {
                        self.declare_var(name.clone(), elem_ty, *is_mut, *span)?;
                    }
                }
                Ok(())
            }
            Stmt::While { condition, body, span } => {
                let cond_ty = self.check_expr(condition)?;
                if cond_ty != Type::Bool {
                    return Err(format!(
                        "Tip Hatası [{}:{}]: 'while' döngü koşulu 'bool' tipinde olmalıdır. Bulunan: '{}'",
                        span.line, span.col, cond_ty
                    ));
                }
                self.loop_depth += 1;
                self.check_block(body)?;
                self.loop_depth -= 1;
                Ok(())
            }
            Stmt::ForRange { var_name, start, end, body, span } => {
                let start_ty = self.check_expr(start)?;
                if start_ty != Type::Int {
                    return Err(format!(
                        "Tip Hatası [{}:{}]: Aralık başlangıcı 'int' olmalıdır, bulunan: '{}'",
                        span.line, span.col, start_ty
                    ));
                }
                let end_ty = self.check_expr(end)?;
                if end_ty != Type::Int {
                    return Err(format!(
                        "Tip Hatası [{}:{}]: Aralık bitişi 'int' olmalıdır, bulunan: '{}'",
                        span.line, span.col, end_ty
                    ));
                }
                self.enter_scope();
                self.declare_var(var_name.clone(), Type::Int, false, *span)?;
                self.loop_depth += 1;
                self.check_block(body)?;
                self.loop_depth -= 1;
                self.exit_scope();
                Ok(())
            }
            Stmt::ForIn { var_name, iterable, body, span } => {
                let iter_ty = self.check_expr(iterable)?;
                let item_ty = match iter_ty {
                    Type::Array(inner) => *inner,
                    Type::Slice(inner) => *inner,
                    Type::Str => Type::Str,
                    Type::StrSlice => Type::Str,
                    other => return Err(format!(
                        "Tip Hatası [{}:{}]: 'for .. in' yalnızca dizi, dilim veya str üzerinde çalışır, bulunan: '{}'",
                        span.line, span.col, other
                    )),
                };
                self.enter_scope();
                self.declare_var(var_name.clone(), item_ty, false, *span)?;
                self.loop_depth += 1;
                self.check_block(body)?;
                self.loop_depth -= 1;
                self.exit_scope();
                Ok(())
            }
            Stmt::Break(span) => {
                if self.loop_depth == 0 {
                    return Err(format!(
                        "Tip Hatası [{}:{}]: 'break' ifadesi yalnızca bir döngü içerisinde kullanılabilir",
                        span.line, span.col
                    ));
                }
                Ok(())
            }
            Stmt::Continue(span) => {
                if self.loop_depth == 0 {
                    return Err(format!(
                        "Tip Hatası [{}:{}]: 'continue' ifadesi yalnızca bir döngü içerisinde kullanılabilir",
                        span.line, span.col
                    ));
                }
                Ok(())
            }
            Stmt::Return(maybe_expr, span) => {
                let ret_ty = if let Some(expr) = maybe_expr {
                    self.check_expr(expr)?
                } else {
                    Type::Void
                };

                if let Some(expected) = &self.current_fn_return {
                    let compatible = if ret_ty == *expected {
                        true
                    } else if let (Type::Option(e_in), Type::Option(r_in)) = (expected, &ret_ty) {
                        **r_in == Type::Void || e_in == r_in
                    } else if let (Type::Result(e_ok, e_err), Type::Result(r_ok, r_err)) = (expected, &ret_ty) {
                        (**r_ok == Type::Void || e_ok == r_ok) && (**r_err == Type::Void || e_err == r_err)
                    } else {
                        false
                    };
                    if !compatible {
                        return Err(format!(
                            "Tip Hatası [{}:{}]: 'return' edilen tip '{}', beklenen dönüş tipi '{}'",
                            span.line, span.col, ret_ty, expected
                        ));
                    }
                }
                Ok(())
            }
            Stmt::Const(c) => {
                let const_val = self.eval_const_expr(&c.value)?;
                if !self.are_types_compatible(&c.ty, &const_val.get_type()) {
                    return Err(format!(
                        "Tip Hatası [{}:{}]: '{}' sabiti '{}' beklerken '{}' hesaplandı",
                        c.span.line, c.span.col, c.name, c.ty, const_val.get_type()
                    ));
                }
                self.constants.insert(c.name.clone(), (c.ty.clone(), const_val));
                self.declare_var(c.name.clone(), c.ty.clone(), false, c.span)?;
                Ok(())
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
        }
    }

    fn eval_const_expr(&self, expr: &Expr) -> Result<ConstValue, String> {
        match expr {
            Expr::Int(val, _) => Ok(ConstValue::Int(*val)),
            Expr::Float(val, _) => Ok(ConstValue::Float(*val)),
            Expr::Bool(val, _) => Ok(ConstValue::Bool(*val)),
            Expr::Str(val, _) => Ok(ConstValue::Str(val.clone())),
            Expr::Char(val, _) => Ok(ConstValue::Char(*val)),
            Expr::U8(val, _) => Ok(ConstValue::U8(*val)),
            Expr::Ident(name, span) => {
                if let Some((_, c_val)) = self.constants.get(name) {
                    Ok(c_val.clone())
                } else {
                    Err(format!("Derleme Hatası [{}:{}]: '{}' bir derleme zamanı sabiti (const) değil", span.line, span.col, name))
                }
            }
            Expr::Binary { op, left, right, span } => {
                let l = self.eval_const_expr(left)?;
                let r = self.eval_const_expr(right)?;
                match (op, l, r) {
                    (BinOp::Add, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a + b)),
                    (BinOp::Sub, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a - b)),
                    (BinOp::Mul, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a * b)),
                    (BinOp::Div, ConstValue::Int(a), ConstValue::Int(b)) => {
                        if b == 0 { return Err(format!("Derleme Hatası [{}:{}]: Derleme zamanında sıfıra bölme hatası", span.line, span.col)); }
                        Ok(ConstValue::Int(a / b))
                    }
                    (BinOp::Mod, ConstValue::Int(a), ConstValue::Int(b)) => {
                        if b == 0 { return Err(format!("Derleme Hatası [{}:{}]: Derleme zamanında sıfıra mod alma hatası", span.line, span.col)); }
                        Ok(ConstValue::Int(a % b))
                    }
                    (BinOp::Add, ConstValue::Float(a), ConstValue::Float(b)) => Ok(ConstValue::Float(a + b)),
                    (BinOp::Sub, ConstValue::Float(a), ConstValue::Float(b)) => Ok(ConstValue::Float(a - b)),
                    (BinOp::Mul, ConstValue::Float(a), ConstValue::Float(b)) => Ok(ConstValue::Float(a * b)),
                    (BinOp::Div, ConstValue::Float(a), ConstValue::Float(b)) => Ok(ConstValue::Float(a / b)),
                    (BinOp::Add, ConstValue::Str(a), ConstValue::Str(b)) => Ok(ConstValue::Str(format!("{}{}", a, b))),
                    (BinOp::Add, ConstValue::Str(a), ConstValue::Int(b)) => Ok(ConstValue::Str(format!("{}{}", a, b))),
                    (BinOp::Add, ConstValue::Int(a), ConstValue::Str(b)) => Ok(ConstValue::Str(format!("{}{}", a, b))),
                    (BinOp::Eq, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a == b)),
                    (BinOp::NotEq, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a != b)),
                    (BinOp::Lt, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a < b)),
                    (BinOp::LtEq, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a <= b)),
                    (BinOp::Gt, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a > b)),
                    (BinOp::GtEq, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a >= b)),
                    (BinOp::And, ConstValue::Bool(a), ConstValue::Bool(b)) => Ok(ConstValue::Bool(a && b)),
                    (BinOp::Or, ConstValue::Bool(a), ConstValue::Bool(b)) => Ok(ConstValue::Bool(a || b)),
                    _ => Err(format!("Derleme Hatası [{}:{}]: Bu ikili işlem sabit ifadelerde desteklenmiyor", span.line, span.col)),
                }
            }
            Expr::Unary { op, expr, span } => {
                let inner = self.eval_const_expr(expr)?;
                match (op, inner) {
                    (UnOp::Neg, ConstValue::Int(a)) => Ok(ConstValue::Int(-a)),
                    (UnOp::Neg, ConstValue::Float(a)) => Ok(ConstValue::Float(-a)),
                    (UnOp::Not, ConstValue::Bool(a)) => Ok(ConstValue::Bool(!a)),
                    _ => Err(format!("Derleme Hatası [{}:{}]: Bu tekil işlem sabit ifadelerde desteklenmiyor", span.line, span.col)),
                }
            }
            other => Err(format!("Derleme Hatası [{}:{}]: İfade derleme zamanı sabiti (const) olarak değerlendirilemez", other.span().line, other.span().col)),
        }
    }

    pub fn check_expr(&mut self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::Int(..) => Ok(Type::Int),
            Expr::Float(..) => Ok(Type::Float),
            Expr::Bool(..) => Ok(Type::Bool),
            Expr::Str(..) => Ok(Type::Str),
            Expr::FormatString(parts, _) => {
                for part in parts {
                    if let FormatPart::Expr(e) = part {
                        self.check_expr(e)?;
                    }
                }
                Ok(Type::Str)
            }
            Expr::Char(..) => Ok(Type::Char),
            Expr::U8(..) => Ok(Type::U8),
            Expr::Ident(name, span) => {
                if name == "None" {
                    return Ok(Type::Option(Box::new(Type::Void)));
                }
                let sym = self.lookup_var(name, *span)?;
                Ok(sym.ty)
            }
            Expr::Unary { op, expr, span } => {
                let ty = self.check_expr(expr)?;
                if let Type::Named(ref s_name) = ty {
                    let m_name = match op {
                        UnOp::Neg => "neg",
                        UnOp::Not => "not",
                    };
                    if let Some(m_fn) = self.methods.get(s_name).and_then(|m| m.get(m_name)) {
                        if m_fn.params.len() == 1 {
                            return Ok(m_fn.return_type.clone());
                        }
                    }
                }
                match op {
                    UnOp::Neg => {
                        if ty == Type::Int || ty == Type::Float {
                            Ok(ty)
                        } else {
                            Err(format!("Tip Hatası [{}:{}]: '-' operatörü yalnızca int veya float ile kullanılabilir, bulunan: '{}'", span.line, span.col, ty))
                        }
                    }
                    UnOp::Not => {
                        if ty == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(format!("Tip Hatası [{}:{}]: '!' operatörü yalnızca bool ile kullanılabilir, bulunan: '{}'", span.line, span.col, ty))
                        }
                    }
                }
            }
            Expr::Binary { op, left, right, span } => {
                let l_ty = self.check_expr(left)?;
                let r_ty = self.check_expr(right)?;

                // Check operator overloading on structs
                let method_name = match op {
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

                if let Type::Named(ref s_name) = l_ty {
                    if let Some(m_name) = method_name {
                        if let Some(m_fn) = self.methods.get(s_name).and_then(|m| m.get(m_name)) {
                            if m_fn.params.len() == 2 {
                                let rhs_param = &m_fn.params[1];
                                if self.are_types_compatible(&rhs_param.ty, &r_ty) {
                                    return Ok(m_fn.return_type.clone());
                                } else {
                                    return Err(format!(
                                        "Tip Hatası [{}:{}]: '{}::{}' operatörü '{}' tipi beklerken '{}' verildi",
                                        span.line, span.col, s_name, m_name, rhs_param.ty, r_ty
                                    ));
                                }
                            }
                        } else if *op == BinOp::NotEq {
                            if let Some(m_fn) = self.methods.get(s_name).and_then(|m| m.get("eq")) {
                                if m_fn.params.len() == 2 {
                                    let rhs_param = &m_fn.params[1];
                                    if self.are_types_compatible(&rhs_param.ty, &r_ty) {
                                        return Ok(Type::Bool);
                                    }
                                }
                            }
                        }
                    }
                }

                match op {
                    BinOp::Add => {
                        // String concatenation if either is str
                        if l_ty == Type::Str || r_ty == Type::Str {
                            Ok(Type::Str)
                        } else if l_ty == Type::Int && r_ty == Type::Int {
                            Ok(Type::Int)
                        } else if l_ty == Type::U8 && r_ty == Type::U8 {
                            Ok(Type::U8)
                        } else if l_ty == Type::Float && r_ty == Type::Float {
                            Ok(Type::Float)
                        } else {
                            Err(format!("Tip Hatası [{}:{}]: '+' operatörü '{}' ve '{}' tipleri arasında uygulanamaz", span.line, span.col, l_ty, r_ty))
                        }
                    }
                    BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if l_ty == Type::Int && r_ty == Type::Int {
                            Ok(Type::Int)
                        } else if l_ty == Type::U8 && r_ty == Type::U8 {
                            Ok(Type::U8)
                        } else if l_ty == Type::Float && r_ty == Type::Float && *op != BinOp::Mod {
                            Ok(Type::Float)
                        } else {
                            Err(format!("Tip Hatası [{}:{}]: Aritmetik işlem '{}' ve '{}' tipleri arasında uygulanamaz", span.line, span.col, l_ty, r_ty))
                        }
                    }
                    BinOp::Eq | BinOp::NotEq => {
                        if l_ty == r_ty || l_ty == Type::Void || r_ty == Type::Void || self.are_types_compatible(&l_ty, &r_ty) || self.are_types_compatible(&r_ty, &l_ty) {
                            Ok(Type::Bool)
                        } else {
                            Err(format!("Tip Hatası [{}:{}]: Karşılaştırma uyumsuz tipler arasında yapılamaz: '{}' ve '{}'", span.line, span.col, l_ty, r_ty))
                        }
                    }
                    BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => {
                        if (l_ty == Type::Int && r_ty == Type::Int)
                            || (l_ty == Type::Float && r_ty == Type::Float)
                            || (l_ty == Type::Char && r_ty == Type::Char)
                            || (l_ty == Type::U8 && r_ty == Type::U8)
                        {
                            Ok(Type::Bool)
                        } else {
                            Err(format!("Tip Hatası [{}:{}]: Sıralama karşılaştırması yalnızca aynı sayı veya karakter tipleri arasında yapılabilir", span.line, span.col))
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        if l_ty == Type::Bool && r_ty == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(format!("Tip Hatası [{}:{}]: Mantıksal operatörler yalnızca 'bool' tipleriyle kullanılabilir", span.line, span.col))
                        }
                    }
                }
            }
            Expr::Assign { target, value, span } => {
                let sym = self.lookup_var(target, *span)?;
                if !sym.is_mut {
                    return Err(format!(
                        "Derleme Hatası [{}:{}]: Değiştirilemez (immutable) '{}' değişkenine yeniden atama yapılamaz! Değiştirmek istiyorsan 'let mut {}' şeklinde tanımlamalısın.",
                        span.line, span.col, target, target
                    ));
                }

                let val_ty = self.check_expr(value)?;
                if sym.ty != val_ty {
                    return Err(format!(
                        "Tip Hatası [{}:{}]: '{}' değişkeni '{}' tipindeyken '{}' tipi atanmaya çalışıldı",
                        span.line, span.col, target, sym.ty, val_ty
                    ));
                }
                Ok(val_ty)
            }
            Expr::Array(elements, span) => {
                if elements.is_empty() {
                    return Ok(Type::Array(Box::new(Type::Void)));
                }

                let first_ty = self.check_expr(&elements[0])?;
                for elem in &elements[1..] {
                    let elem_ty = self.check_expr(elem)?;
                    if elem_ty != first_ty {
                        return Err(format!(
                            "Tip Hatası [{}:{}]: Dizi elemanları homojen (aynı tipte) olmalıdır. Beklenen: '{}', Bulunan: '{}'",
                            span.line, span.col, first_ty, elem_ty
                        ));
                    }
                }
                Ok(Type::Array(Box::new(first_ty)))
            }
            Expr::Tuple(elements, _) => {
                let mut elem_tys = Vec::new();
                for e in elements {
                    elem_tys.push(self.check_expr(e)?);
                }
                Ok(Type::Tuple(elem_tys))
            }
            Expr::TupleIndex { target, index, span } => {
                let target_ty = self.check_expr(target)?;
                match target_ty {
                    Type::Tuple(elem_tys) => {
                        if *index >= elem_tys.len() {
                            return Err(format!(
                                "Tip Hatası [{}:{}]: Demet indeksi sınırların dışında: demet {} elemanlı, ancak indeks {} istendi",
                                span.line, span.col, elem_tys.len(), index
                            ));
                        }
                        Ok(elem_tys[*index].clone())
                    }
                    _ => Err(format!(
                        "Tip Hatası [{}:{}]: Demet indeksi '.' operatörü sadece demet tipleri üzerinde kullanılabilir, bulunan: '{}'",
                        span.line, span.col, target_ty
                    )),
                }
            }
            Expr::TupleIndexAssign { target, index, value, span } => {
                if let Expr::Ident(name, ident_span) = &**target {
                    let sym = self.lookup_var(name, *ident_span)?;
                    if !sym.is_mut {
                        return Err(format!(
                            "Derleme Hatası [{}:{}]: Değiştirilemez (immutable) '{}' demetinin elemanları değiştirilemez! 'let mut {}' şeklinde tanımlamalısın.",
                            ident_span.line, ident_span.col, name, name
                        ));
                    }
                }
                let target_ty = self.check_expr(target)?;
                let val_ty = self.check_expr(value)?;
                match target_ty {
                    Type::Tuple(elem_tys) => {
                        if *index >= elem_tys.len() {
                            return Err(format!(
                                "Tip Hatası [{}:{}]: Demet indeksi sınırların dışında: demet {} elemanlı, ancak indeks {} istendi",
                                span.line, span.col, elem_tys.len(), index
                            ));
                        }
                        if !self.are_types_compatible(&elem_tys[*index], &val_ty) {
                            return Err(format!(
                                "Tip Hatası [{}:{}]: Demetin {}. elemanına '{}' tipi atanamaz (beklenen: '{}')",
                                span.line, span.col, index, val_ty, elem_tys[*index]
                            ));
                        }
                        Ok(val_ty)
                    }
                    _ => Err(format!(
                        "Tip Hatası [{}:{}]: Demet indeksi '.' operatörü sadece demet tipleri üzerinde kullanılabilir, bulunan: '{}'",
                        span.line, span.col, target_ty
                    )),
                }
            }
            Expr::Index { target, index, span } => {
                let target_ty = self.check_expr(target)?;
                let index_ty = self.check_expr(index)?;

                match target_ty {
                    Type::Array(inner) => {
                        if index_ty != Type::Int {
                            return Err(format!("Tip Hatası [{}:{}]: Dizi indeksi 'int' olmalıdır, bulunan: '{}'", span.line, span.col, index_ty));
                        }
                        Ok(*inner)
                    }
                    Type::Slice(inner) => {
                        if index_ty != Type::Int {
                            return Err(format!("Tip Hatası [{}:{}]: Dilim indeksi 'int' olmalıdır, bulunan: '{}'", span.line, span.col, index_ty));
                        }
                        Ok(*inner)
                    }
                    Type::Str | Type::StrSlice => {
                        if index_ty != Type::Int {
                            return Err(format!("Tip Hatası [{}:{}]: Metin indeksi 'int' olmalıdır, bulunan: '{}'", span.line, span.col, index_ty));
                        }
                        Ok(Type::Str)
                    }
                    Type::Named(ref s_name) => {
                        if let Some(m_fn) = self.methods.get(s_name).and_then(|m| m.get("index")) {
                            if m_fn.params.len() == 2 && self.are_types_compatible(&m_fn.params[1].ty, &index_ty) {
                                return Ok(m_fn.return_type.clone());
                            } else if m_fn.params.len() == 2 {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: '{}::index' metodu '{}' indeks tipi beklerken '{}' verildi",
                                    span.line, span.col, s_name, m_fn.params[1].ty, index_ty
                                ));
                            }
                        }
                        Err(format!("Tip Hatası [{}:{}]: '{}' tipi indekslenemez (Index trait'i veya index(self, idx) metodu tanımlanmamış)", span.line, span.col, s_name))
                    }
                    other => Err(format!("Tip Hatası [{}:{}]: '{}' tipi indekslenemez (yalnızca dizi, dilim veya str indekslenebilir)", span.line, span.col, other)),
                }
            }
            Expr::Slice { target, start, end, span } => {
                let target_ty = self.check_expr(target)?;
                if let Some(s) = start {
                    let s_ty = self.check_expr(s)?;
                    if s_ty != Type::Int {
                        return Err(format!("Tip Hatası [{}:{}]: Dilim başlangıç indeksi 'int' olmalıdır, bulunan: '{}'", span.line, span.col, s_ty));
                    }
                }
                if let Some(e) = end {
                    let e_ty = self.check_expr(e)?;
                    if e_ty != Type::Int {
                        return Err(format!("Tip Hatası [{}:{}]: Dilim bitiş indeksi 'int' olmalıdır, bulunan: '{}'", span.line, span.col, e_ty));
                    }
                }

                match target_ty {
                    Type::Array(inner) => Ok(Type::Slice(inner)),
                    Type::Slice(inner) => Ok(Type::Slice(inner)),
                    Type::Str => Ok(Type::StrSlice),
                    Type::StrSlice => Ok(Type::StrSlice),
                    other => Err(format!(
                        "Tip Hatası [{}:{}]: '{}' tipi dilimlenemez (yalnızca dizi, dilim veya metin dilimlenebilir)",
                        span.line, span.col, other
                    )),
                }
            }
            Expr::IndexAssign { target, index, value, span } => {
                if let Expr::Ident(name, ident_span) = &**target {
                    let sym = self.lookup_var(name, *ident_span)?;
                    if !sym.is_mut {
                        return Err(format!(
                            "Derleme Hatası [{}:{}]: Değiştirilemez (immutable) '{}' dizisinin elemanları değiştirilemez! 'let mut {}' şeklinde tanımlamalısın.",
                            ident_span.line, ident_span.col, name, name
                        ));
                    }
                }

                let target_ty = self.check_expr(target)?;
                let index_ty = self.check_expr(index)?;
                let val_ty = self.check_expr(value)?;

                match target_ty {
                    Type::Array(inner) => {
                        if index_ty != Type::Int {
                            return Err(format!("Tip Hatası [{}:{}]: Dizi indeksi 'int' olmalıdır", span.line, span.col));
                        }
                        if *inner != val_ty {
                            return Err(format!("Tip Hatası [{}:{}]: '{}' tipindeki diziye '{}' tipi atanamaz", span.line, span.col, inner, val_ty));
                        }
                        Ok(val_ty)
                    }
                    Type::Named(ref s_name) => {
                        if let Some(m_fn) = self.methods.get(s_name).and_then(|m| m.get("index_set")) {
                            if m_fn.params.len() == 3
                                && self.are_types_compatible(&m_fn.params[1].ty, &index_ty)
                                && self.are_types_compatible(&m_fn.params[2].ty, &val_ty)
                            {
                                return Ok(val_ty);
                            } else if m_fn.params.len() == 3 {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: '{}::index_set' metodu ({}, {}) parametreleri beklerken ({}, {}) verildi",
                                    span.line, span.col, s_name, m_fn.params[1].ty, m_fn.params[2].ty, index_ty, val_ty
                                ));
                            }
                        }
                        Err(format!("Tip Hatası [{}:{}]: '{}' tipi indeks ataması desteklemiyor (index_set(self, idx, val) metodu tanımlanmamış)", span.line, span.col, s_name))
                    }
                    _ => Err(format!("Tip Hatası [{}:{}]: Yalnızca dizilerin veya 'index_set' uygulayan nesnelerin indekslerine eleman atanabilir", span.line, span.col)),
                }
            }
            Expr::StructInit { name, fields, span, .. } => {
                let s_def = match self.structs.get(name) {
                    Some(s) => s.clone(),
                    None => return Err(format!("Tip Hatası [{}:{}]: Tanımsız struct '{}'", span.line, span.col, name)),
                };

                let mut field_names: HashSet<String> = HashSet::new();
                for (f_name, f_val) in fields {
                    field_names.insert(f_name.clone());
                    let expected_ty = s_def.fields.iter().find(|(k, _)| k == f_name).map(|(_, ty)| ty);
                    match expected_ty {
                        Some(exp) => {
                            let actual_ty = self.check_expr(f_val)?;
                            if !self.are_types_compatible(exp, &actual_ty) {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: '{}' struct'ının '{}' alanı '{}' beklerken '{}' verildi",
                                    span.line, span.col, name, f_name, exp, actual_ty
                                ));
                            }
                        }
                        None => {
                            return Err(format!("Tip Hatası [{}:{}]: '{}' struct'ında '{}' adında bir alan yok", span.line, span.col, name, f_name));
                        }
                    }
                }

                if field_names.len() != s_def.fields.len() {
                    return Err(format!("Tip Hatası [{}:{}]: '{}' struct başlatılırken bazı alanlar eksik bırakılmış", span.line, span.col, name));
                }

                Ok(Type::Named(name.clone()))
            }
            Expr::FieldAccess { target, field, span } => {
                let target_ty = self.check_expr(target)?;
                let mut curr_ty = target_ty;
                while let Type::Box(in_t) | Type::Rc(in_t) | Type::RefCell(in_t) = curr_ty {
                    curr_ty = *in_t;
                }
                match curr_ty {
                    Type::Named(s_name) => {
                        let s_def = match self.structs.get(&s_name) {
                            Some(s) => s,
                            None => return Err(format!("Tip Hatası [{}:{}]: Tanımsız tip '{}'", span.line, span.col, s_name)),
                        };

                        if let Some((_, f_ty)) = s_def.fields.iter().find(|(k, _)| k == field) {
                            Ok(f_ty.clone())
                        } else {
                            Err(format!("Tip Hatası [{}:{}]: '{}' struct'ında '{}' alanı bulunmuyor", span.line, span.col, s_name, field))
                        }
                    }
                    other => Err(format!("Tip Hatası [{}:{}]: '{}' tipinde alan erişimi (.) yapılamaz", span.line, span.col, other)),
                }
            }
            Expr::FieldAssign { target, field, value, span } => {
                if let Expr::Ident(name, ident_span) = &**target {
                    let sym = self.lookup_var(name, *ident_span)?;
                    if !sym.is_mut {
                        return Err(format!(
                            "Derleme Hatası [{}:{}]: Değiştirilemez (immutable) '{}' nesnesinin alanları değiştirilemez! 'let mut {}' şeklinde tanımlamalısın.",
                            ident_span.line, ident_span.col, name, name
                        ));
                    }
                }

                let target_ty = self.check_expr(target)?;
                let val_ty = self.check_expr(value)?;

                let mut curr_ty = target_ty;
                while let Type::Box(in_t) = curr_ty {
                    curr_ty = *in_t;
                }

                match curr_ty {
                    Type::Named(s_name) => {
                        let s_def = match self.structs.get(&s_name) {
                            Some(s) => s,
                            None => return Err(format!("Tip Hatası [{}:{}]: Tanımsız struct '{}'", span.line, span.col, s_name)),
                        };

                        if let Some((_, f_ty)) = s_def.fields.iter().find(|(k, _)| k == field) {
                            if *f_ty != val_ty {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: '{}' struct'ının '{}' alanına '{}' tipi atanamaz (beklenen: '{}')",
                                    span.line, span.col, s_name, field, val_ty, f_ty
                                ));
                            }
                            Ok(val_ty)
                        } else {
                            Err(format!("Tip Hatası [{}:{}]: '{}' struct'ında '{}' alanı bulunamadı", span.line, span.col, s_name, field))
                        }
                    }
                    _ => Err(format!("Tip Hatası [{}:{}]: Yalnızca struct alanlarına atama yapılabilir", span.line, span.col)),
                }
            }
            Expr::MethodCall { target, method, args, span, .. } => {
                let target_ty = self.check_expr(target)?;
                match target_ty {
                    Type::Option(inner) => {
                        match method.as_str() {
                            "is_some" | "is_none" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: '{}' argüman beklemez", span.line, span.col, method)); }
                                return Ok(Type::Bool);
                            }
                            "unwrap" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'unwrap()' argüman beklemez", span.line, span.col)); }
                                return Ok(*inner);
                            }
                            "unwrap_or" => {
                                if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'unwrap_or()' 1 varsayılan değer bekler", span.line, span.col)); }
                                let def_ty = self.check_expr(&args[0])?;
                                if def_ty != *inner && *inner != Type::Void {
                                    return Err(format!("Tip Hatası [{}:{}]: 'unwrap_or()' varsayılan değeri '{}' beklerken '{}' bulundu", span.line, span.col, inner, def_ty));
                                }
                                return Ok(*inner);
                            }
                            _ => return Err(format!("Tip Hatası [{}:{}]: 'Option' için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Type::Result(ok, _err) => {
                        match method.as_str() {
                            "is_ok" | "is_err" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: '{}' argüman beklemez", span.line, span.col, method)); }
                                return Ok(Type::Bool);
                            }
                            "unwrap" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'unwrap()' argüman beklemez", span.line, span.col)); }
                                return Ok(*ok);
                            }
                            "unwrap_or" => {
                                if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'unwrap_or()' 1 varsayılan değer bekler", span.line, span.col)); }
                                let def_ty = self.check_expr(&args[0])?;
                                if def_ty != *ok && *ok != Type::Void {
                                    return Err(format!("Tip Hatası [{}:{}]: 'unwrap_or()' varsayılan değeri '{}' beklerken '{}' bulundu", span.line, span.col, ok, def_ty));
                                }
                                return Ok(*ok);
                            }
                            _ => return Err(format!("Tip Hatası [{}:{}]: 'Result' için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Type::Str => {
                        match method.as_str() {
                            "len" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'len()' argüman beklemez", span.line, span.col)); }
                                return Ok(Type::Int);
                            }
                            "trim" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'trim()' argüman beklemez", span.line, span.col)); }
                                return Ok(Type::Str);
                            }
                            "to_upper" | "to_uppercase" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'to_upper()' argüman beklemez", span.line, span.col)); }
                                return Ok(Type::Str);
                            }
                            "to_lower" | "to_lowercase" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'to_lower()' argüman beklemez", span.line, span.col)); }
                                return Ok(Type::Str);
                            }
                            "contains" => {
                                if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'contains()' 1 argüman bekler", span.line, span.col)); }
                                let a_ty = self.check_expr(&args[0])?;
                                if a_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'contains()' 'str' bekler", span.line, span.col)); }
                                return Ok(Type::Bool);
                            }
                            "split" => {
                                if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'split()' 1 argüman (ayraç) bekler", span.line, span.col)); }
                                let a_ty = self.check_expr(&args[0])?;
                                if a_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'split()' 'str' ayraç bekler", span.line, span.col)); }
                                return Ok(Type::Array(Box::new(Type::Str)));
                            }
                            "replace" => {
                                if args.len() != 2 { return Err(format!("Tip Hatası [{}:{}]: 'replace()' 2 argüman bekler: s.replace(eski, yeni)", span.line, span.col)); }
                                let from_ty = self.check_expr(&args[0])?;
                                let to_ty = self.check_expr(&args[1])?;
                                if from_ty != Type::Str || to_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'replace()' 'str' argümanlar bekler", span.line, span.col)); }
                                return Ok(Type::Str);
                            }
                            "char_at" => {
                                if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'char_at()' 1 indeks bekler", span.line, span.col)); }
                                let idx_ty = self.check_expr(&args[0])?;
                                if idx_ty != Type::Int { return Err(format!("Tip Hatası [{}:{}]: 'char_at()' 'int' indeks bekler", span.line, span.col)); }
                                return Ok(Type::Char);
                            }
                            _ => return Err(format!("Tip Hatası [{}:{}]: 'str' tipi için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Type::Array(ref inner) => {
                        match method.as_str() {
                            "len" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'len()' argüman beklemez", span.line, span.col)); }
                                return Ok(Type::Int);
                            }
                            "push" => {
                                if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'push()' 1 argüman bekler", span.line, span.col)); }
                                let elem_ty = self.check_expr(&args[0])?;
                                if **inner != elem_ty && **inner != Type::Void {
                                    return Err(format!("Tip Hatası [{}:{}]: Dizi '{}' tipindeyken 'push' ile '{}' tipi eklenemez", span.line, span.col, inner, elem_ty));
                                }
                                return Ok(Type::Void);
                            }
                            "pop" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'pop()' argüman beklemez", span.line, span.col)); }
                                return Ok((**inner).clone());
                            }
                            "map" => {
                                if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'map()' 1 fonksiyon/closure argümanı bekler", span.line, span.col)); }
                                let fn_ty = self.check_expr(&args[0])?;
                                match fn_ty {
                                    Type::Fn { params, ret } => {
                                        if params.len() != 1 {
                                            return Err(format!("Tip Hatası [{}:{}]: 'map' fonksiyonu 1 parametre bekler", span.line, span.col));
                                        }
                                        return Ok(Type::Array(ret));
                                    }
                                    _ => return Err(format!("Tip Hatası [{}:{}]: 'map' argümanı fonksiyon veya closure olmalıdır", span.line, span.col)),
                                }
                            }
                            "filter" => {
                                if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'filter()' 1 fonksiyon/closure argümanı bekler", span.line, span.col)); }
                                let fn_ty = self.check_expr(&args[0])?;
                                match fn_ty {
                                    Type::Fn { params, ret } => {
                                        if params.len() != 1 {
                                            return Err(format!("Tip Hatası [{}:{}]: 'filter' fonksiyonu 1 parametre bekler", span.line, span.col));
                                        }
                                        if *ret != Type::Bool {
                                            return Err(format!("Tip Hatası [{}:{}]: 'filter' fonksiyonu 'bool' değer dönmelidir", span.line, span.col));
                                        }
                                        return Ok(Type::Array(inner.clone()));
                                    }
                                    _ => return Err(format!("Tip Hatası [{}:{}]: 'filter' argümanı bool dönen bir fonksiyon veya closure olmalıdır", span.line, span.col)),
                                }
                            }
                            _ => return Err(format!("Tip Hatası [{}:{}]: Dizi için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Type::Slice(ref inner) => {
                        match method.as_str() {
                            "len" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'len()' argüman beklemez", span.line, span.col)); }
                                return Ok(Type::Int);
                            }
                            "to_array" | "to_vec" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'to_array()' argüman beklemez", span.line, span.col)); }
                                return Ok(Type::Array(inner.clone()));
                            }
                            _ => return Err(format!("Tip Hatası [{}:{}]: Dilim için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Type::StrSlice => {
                        match method.as_str() {
                            "len" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'len()' argüman beklemez", span.line, span.col)); }
                                return Ok(Type::Int);
                            }
                            "to_str" | "to_string" => {
                                if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'to_str()' argüman beklemez", span.line, span.col)); }
                                return Ok(Type::Str);
                            }
                            "trim" => return Ok(Type::Str),
                            "to_upper" | "to_uppercase" => return Ok(Type::Str),
                            "to_lower" | "to_lowercase" => return Ok(Type::Str),
                            "contains" => return Ok(Type::Bool),
                            "char_at" => return Ok(Type::Char),
                            _ => return Err(format!("Tip Hatası [{}:{}]: '&str' için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Type::Named(ref struct_name) => {
                        if struct_name == "Map" {
                            match method.as_str() {
                                "set" => {
                                    if args.len() != 2 { return Err(format!("Tip Hatası [{}:{}]: 'Map.set()' 2 argüman bekler: (anahtar: str, deger)", span.line, span.col)); }
                                    let k_ty = self.check_expr(&args[0])?;
                                    if k_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'Map.set()' anahtarı 'str' olmalıdır", span.line, span.col)); }
                                    self.check_expr(&args[1])?;
                                    return Ok(Type::Void);
                                }
                                "get" => {
                                    if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'Map.get()' 1 argüman bekler: (anahtar: str)", span.line, span.col)); }
                                    let k_ty = self.check_expr(&args[0])?;
                                    if k_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'Map.get()' anahtarı 'str' olmalıdır", span.line, span.col)); }
                                    return Ok(Type::Str);
                                }
                                "has" => {
                                    if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'Map.has()' 1 argüman bekler: (anahtar: str)", span.line, span.col)); }
                                    let k_ty = self.check_expr(&args[0])?;
                                    if k_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'Map.has()' anahtarı 'str' olmalıdır", span.line, span.col)); }
                                    return Ok(Type::Bool);
                                }
                                "len" => {
                                    if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'Map.len()' argüman beklemez", span.line, span.col)); }
                                    return Ok(Type::Int);
                                }
                                "keys" => {
                                    if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'Map.keys()' argüman beklemez", span.line, span.col)); }
                                    return Ok(Type::Array(Box::new(Type::Str)));
                                }
                                _ => return Err(format!("Tip Hatası [{}:{}]: 'Map' için '{}' metodu bulunamadı", span.line, span.col, method)),
                            }
                        }

                        let method_fn = match self.methods.get(struct_name).and_then(|m| m.get(method)) {
                            Some(f) => f.clone(),
                            None => return Err(format!("Tip Hatası [{}:{}]: '{}' struct'ı için '{}' metodu tanımlanmamış", span.line, span.col, struct_name, method)),
                        };

                        // Check self parameter
                        let has_self = method_fn.params.first().map_or(false, |p| p.name == "self");
                        let expected_arg_count = if has_self { method_fn.params.len() - 1 } else { method_fn.params.len() };

                        if args.len() != expected_arg_count {
                            return Err(format!(
                                "Tip Hatası [{}:{}]: '{}::{}' metodu {} argüman beklerken {} argüman verildi",
                                span.line, span.col, struct_name, method, expected_arg_count, args.len()
                            ));
                        }

                        let param_offset = if has_self { 1 } else { 0 };
                        for (idx, arg_expr) in args.iter().enumerate() {
                            let arg_ty = self.check_expr(arg_expr)?;
                            let expected_param = &method_fn.params[idx + param_offset];
                            if !self.are_types_compatible(&expected_param.ty, &arg_ty) {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: '{}::{}' metodunun {}. argümanı ('{}') için '{}' tipi beklenirken '{}' verildi",
                                    span.line, span.col, struct_name, method, idx + 1, expected_param.name, expected_param.ty, arg_ty
                                ));
                            }
                        }

                        Ok(method_fn.return_type)
                    }
                    Type::Rc(inner) if method == "clone" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'clone()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Rc(inner));
                    }
                    Type::RefCell(inner) if method == "borrow" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'borrow()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Ref { inner, is_mut: false });
                    }
                    Type::RefCell(inner) if method == "borrow_mut" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'borrow_mut()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Ref { inner, is_mut: true });
                    }
                    Type::JoinHandle(inner) if method == "join" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'join()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Result(inner, Box::new(Type::Str)));
                    }
                    Type::Sender(inner) if method == "send" => {
                        if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'send' 1 argüman bekler", span.line, span.col)); }
                        let arg_ty = self.check_expr(&args[0])?;
                        if *inner != Type::Void && !self.are_types_compatible(&inner, &arg_ty) {
                            return Err(format!("Tip Hatası [{}:{}]: 'send' '{}' tipi beklerken '{}' verildi", span.line, span.col, inner, arg_ty));
                        }
                        return Ok(Type::Result(Box::new(Type::Void), Box::new(Type::Str)));
                    }
                    Type::Receiver(inner) if method == "recv" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'recv()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Result(inner, Box::new(Type::Str)));
                    }
                    Type::Mutex(inner) if method == "lock" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'lock()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Ref { inner, is_mut: true });
                    }
                    Type::TcpListener if method == "accept" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'accept()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Result(Box::new(Type::TcpStream), Box::new(Type::Str)));
                    }
                    Type::TcpStream if method == "write" => {
                        if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'write' 1 argüman bekler", span.line, span.col)); }
                        let arg_ty = self.check_expr(&args[0])?;
                        if !self.are_types_compatible(&Type::Str, &arg_ty) {
                            return Err(format!("Tip Hatası [{}:{}]: 'write' 'str' tipi beklerken '{}' verildi", span.line, span.col, arg_ty));
                        }
                        return Ok(Type::Result(Box::new(Type::Int), Box::new(Type::Str)));
                    }
                    Type::TcpStream if method == "read" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'read()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Result(Box::new(Type::Str), Box::new(Type::Str)));
                    }
                    Type::TcpStream if method == "close" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'close()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Void);
                    }
                    Type::Box(ref inner) | Type::Rc(ref inner) => {
                        let mut curr_inner = (**inner).clone();
                        while let Type::Box(in_t) | Type::Rc(in_t) = curr_inner {
                            curr_inner = *in_t;
                        }
                        if let Type::Named(ref s_name) = curr_inner {
                            let method_fn = match self.methods.get(s_name).and_then(|m| m.get(method)) {
                                Some(f) => f.clone(),
                                None => return Err(format!("Tip Hatası [{}:{}]: '{}' struct'ı için '{}' metodu tanımlanmamış", span.line, span.col, s_name, method)),
                            };
                            let has_self = method_fn.params.first().map_or(false, |p| p.name == "self");
                            let expected_arg_count = if has_self { method_fn.params.len() - 1 } else { method_fn.params.len() };
                            if args.len() != expected_arg_count {
                                return Err(format!("Tip Hatası [{}:{}]: '{}::{}' metodu {} argüman beklerken {} verildi", span.line, span.col, s_name, method, expected_arg_count, args.len()));
                            }
                            let param_offset = if has_self { 1 } else { 0 };
                            for (idx, arg_expr) in args.iter().enumerate() {
                                let arg_ty = self.check_expr(arg_expr)?;
                                let expected_param = &method_fn.params[idx + param_offset];
                                if !self.are_types_compatible(&expected_param.ty, &arg_ty) {
                                    return Err(format!("Tip Hatası [{}:{}]: '{}::{}' metodunun {}. argümanı ('{}') için '{}' beklenirken '{}' verildi", span.line, span.col, s_name, method, idx + 1, expected_param.name, expected_param.ty, arg_ty));
                                }
                            }
                            return Ok(method_fn.return_type);
                        } else {
                            return Err(format!("Tip Hatası [{}:{}]: 'Box<{}>' tipi için '{}' metodu bulunamadı", span.line, span.col, inner, method));
                        }
                    }
                    other => Err(format!("Tip Hatası [{}:{}]: '{}' tipi üzerinde metot çağrılamaz", span.line, span.col, other)),
                }
            }
            Expr::Call { callee, args, span, .. } => {
                match callee.as_str() {
                    "thread::spawn" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'thread::spawn' tam 1 closure/fonksiyon bekler", span.line, span.col));
                        }
                        let callable_ty = self.check_expr(&args[0])?;
                        let ret_ty = match callable_ty {
                            Type::Fn { ret, .. } => *ret,
                            other => return Err(format!("Tip Hatası [{}:{}]: 'thread::spawn' fonksiyon veya closure bekler, bulunan: '{}'", span.line, span.col, other)),
                        };
                        return Ok(Type::JoinHandle(Box::new(ret_ty)));
                    }
                    "thread::sleep" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'thread::sleep' 1 argüman bekler: (ms: int)", span.line, span.col));
                        }
                        let ms_ty = self.check_expr(&args[0])?;
                        if ms_ty != Type::Int {
                            return Err(format!("Tip Hatası [{}:{}]: 'thread::sleep' bekleme süresini 'int' bekler", span.line, span.col));
                        }
                        return Ok(Type::Void);
                    }
                    "channel" => {
                        if !args.is_empty() {
                            return Err(format!("Tip Hatası [{}:{}]: 'channel' argüman beklemez: channel()", span.line, span.col));
                        }
                        return Ok(Type::Tuple(vec![
                            Type::Sender(Box::new(Type::Void)),
                            Type::Receiver(Box::new(Type::Void)),
                        ]));
                    }
                    "Mutex::new" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'Mutex::new' tam 1 argüman bekler", span.line, span.col));
                        }
                        let inner_ty = self.check_expr(&args[0])?;
                        return Ok(Type::Mutex(Box::new(inner_ty)));
                    }
                    "TcpListener::bind" | "net::TcpListener::bind" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'TcpListener::bind' 1 argüman bekler: (addr: str)", span.line, span.col));
                        }
                        let addr_ty = self.check_expr(&args[0])?;
                        if !self.are_types_compatible(&Type::Str, &addr_ty) {
                            return Err(format!("Tip Hatası [{}:{}]: 'TcpListener::bind' adres parametresini 'str' bekler", span.line, span.col));
                        }
                        return Ok(Type::Result(Box::new(Type::TcpListener), Box::new(Type::Str)));
                    }
                    "TcpStream::connect" | "net::TcpStream::connect" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'TcpStream::connect' 1 argüman bekler: (addr: str)", span.line, span.col));
                        }
                        let addr_ty = self.check_expr(&args[0])?;
                        if !self.are_types_compatible(&Type::Str, &addr_ty) {
                            return Err(format!("Tip Hatası [{}:{}]: 'TcpStream::connect' adres parametresini 'str' bekler", span.line, span.col));
                        }
                        return Ok(Type::Result(Box::new(Type::TcpStream), Box::new(Type::Str)));
                    }
                    "Box::new" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'Box::new' tam 1 argüman bekler", span.line, span.col));
                        }
                        let inner_ty = self.check_expr(&args[0])?;
                        return Ok(Type::Box(Box::new(inner_ty)));
                    }
                    "Rc::new" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'Rc::new' tam 1 argüman bekler", span.line, span.col));
                        }
                        let inner_ty = self.check_expr(&args[0])?;
                        return Ok(Type::Rc(Box::new(inner_ty)));
                    }
                    "Rc::strong_count" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'Rc::strong_count' tam 1 argüman bekler", span.line, span.col));
                        }
                        let arg_ty = self.check_expr(&args[0])?;
                        if !matches!(arg_ty, Type::Rc(_)) {
                            return Err(format!("Tip Hatası [{}:{}]: 'Rc::strong_count' bir Rc<T> nesnesi bekler, bulunan: '{}'", span.line, span.col, arg_ty));
                        }
                        return Ok(Type::Int);
                    }
                    "RefCell::new" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'RefCell::new' tam 1 argüman bekler", span.line, span.col));
                        }
                        let inner_ty = self.check_expr(&args[0])?;
                        return Ok(Type::RefCell(Box::new(inner_ty)));
                    }
                    "Some" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'Some' 1 argüman bekler", span.line, span.col));
                        }
                        let inner_ty = self.check_expr(&args[0])?;
                        return Ok(Type::Option(Box::new(inner_ty)));
                    }
                    "Ok" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'Ok' 1 argüman bekler", span.line, span.col));
                        }
                        let inner_ty = self.check_expr(&args[0])?;
                        return Ok(Type::Result(Box::new(inner_ty), Box::new(Type::Void)));
                    }
                    "Err" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'Err' 1 argüman bekler", span.line, span.col));
                        }
                        let inner_ty = self.check_expr(&args[0])?;
                        return Ok(Type::Result(Box::new(Type::Void), Box::new(inner_ty)));
                    }
                    "assert" => {
                        if args.is_empty() || args.len() > 2 {
                            return Err(format!("Tip Hatası [{}:{}]: 'assert' 1 veya 2 argüman bekler: assert(kosul, [mesaj])", span.line, span.col));
                        }
                        let cond_ty = self.check_expr(&args[0])?;
                        if cond_ty != Type::Bool {
                            return Err(format!("Tip Hatası [{}:{}]: 'assert' ilk argüman olarak 'bool' bekler, bulunan: '{}'", span.line, span.col, cond_ty));
                        }
                        if args.len() == 2 {
                            let msg_ty = self.check_expr(&args[1])?;
                            if msg_ty != Type::Str {
                                return Err(format!("Tip Hatası [{}:{}]: 'assert' mesajı 'str' olmalıdır, bulunan: '{}'", span.line, span.col, msg_ty));
                            }
                        }
                        return Ok(Type::Void);
                    }
                    "println" | "print" => {
                        for a in args {
                            self.check_expr(a)?;
                        }
                        return Ok(Type::Void);
                    }
                    "len" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'len' fonksiyonu 1 argüman bekler", span.line, span.col));
                        }
                        let a_ty = self.check_expr(&args[0])?;
                        match a_ty {
                            Type::Array(_) | Type::Slice(_) | Type::Str | Type::StrSlice => return Ok(Type::Int),
                            _ => return Err(format!("Tip Hatası [{}:{}]: 'len' yalnızca dizi, dilim veya metinler için kullanılabilir", span.line, span.col)),
                        }
                    }
                    "push" => {
                        if args.len() != 2 {
                            return Err(format!("Tip Hatası [{}:{}]: 'push' fonksiyonu 2 argüman bekler: push(dizi, eleman)", span.line, span.col));
                        }
                        let arr_ty = self.check_expr(&args[0])?;
                        let elem_ty = self.check_expr(&args[1])?;
                        match arr_ty {
                            Type::Array(inner) => {
                                if *inner != elem_ty && *inner != Type::Void {
                                    return Err(format!("Tip Hatası [{}:{}]: Dizi '{}' tipindeyken 'push' ile '{}' tipi eklenemez", span.line, span.col, inner, elem_ty));
                                }
                                return Ok(Type::Void);
                            }
                            _ => return Err(format!("Tip Hatası [{}:{}]: 'push' ilk argüman olarak dizi bekler", span.line, span.col)),
                        }
                    }
                    "pop" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'pop' fonksiyonu 1 argüman bekler", span.line, span.col));
                        }
                        let arr_ty = self.check_expr(&args[0])?;
                        match arr_ty {
                            Type::Array(inner) => return Ok(*inner),
                            _ => return Err(format!("Tip Hatası [{}:{}]: 'pop' dizi bekler", span.line, span.col)),
                        }
                    }
                    "read_file" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'read_file' fonksiyonu 1 argüman bekler: read_file(yol: str)", span.line, span.col));
                        }
                        let path_ty = self.check_expr(&args[0])?;
                        if path_ty != Type::Str {
                            return Err(format!("Tip Hatası [{}:{}]: 'read_file' dosya yolunu 'str' olarak bekler, bulunan: '{}'", span.line, span.col, path_ty));
                        }
                        return Ok(Type::Str);
                    }
                    "fs::read_to_string" | "read_to_string" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'fs::read_to_string' 1 argüman bekler: (yol: str)", span.line, span.col));
                        }
                        let path_ty = self.check_expr(&args[0])?;
                        if path_ty != Type::Str {
                            return Err(format!("Tip Hatası [{}:{}]: 'fs::read_to_string' dosya yolunu 'str' olarak bekler, bulunan: '{}'", span.line, span.col, path_ty));
                        }
                        return Ok(Type::Result(Box::new(Type::Str), Box::new(Type::Str)));
                    }
                    "fs::write" => {
                        if args.len() != 2 {
                            return Err(format!("Tip Hatası [{}:{}]: 'fs::write' 2 argüman bekler: (yol: str, icerik: str)", span.line, span.col));
                        }
                        let path_ty = self.check_expr(&args[0])?;
                        let content_ty = self.check_expr(&args[1])?;
                        if path_ty != Type::Str || content_ty != Type::Str {
                            return Err(format!("Tip Hatası [{}:{}]: 'fs::write' (yol: str, icerik: str) bekler", span.line, span.col));
                        }
                        return Ok(Type::Result(Box::new(Type::Void), Box::new(Type::Str)));
                    }
                    "fs::remove_file" | "remove_file" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'fs::remove_file' 1 argüman bekler: (yol: str)", span.line, span.col));
                        }
                        let path_ty = self.check_expr(&args[0])?;
                        if path_ty != Type::Str {
                            return Err(format!("Tip Hatası [{}:{}]: 'fs::remove_file' dosya yolunu 'str' olarak bekler, bulunan: '{}'", span.line, span.col, path_ty));
                        }
                        return Ok(Type::Result(Box::new(Type::Void), Box::new(Type::Str)));
                    }
                    "fs::exists" | "file_exists" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'fs::exists' 1 argüman bekler: (yol: str)", span.line, span.col));
                        }
                        let path_ty = self.check_expr(&args[0])?;
                        if path_ty != Type::Str {
                            return Err(format!("Tip Hatası [{}:{}]: 'fs::exists' dosya yolunu 'str' olarak bekler, bulunan: '{}'", span.line, span.col, path_ty));
                        }
                        return Ok(Type::Bool);
                    }
                    "fs::create_dir" | "create_dir" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'fs::create_dir' 1 argüman bekler: (yol: str)", span.line, span.col));
                        }
                        let path_ty = self.check_expr(&args[0])?;
                        if path_ty != Type::Str {
                            return Err(format!("Tip Hatası [{}:{}]: 'fs::create_dir' yolunu 'str' olarak bekler, bulunan: '{}'", span.line, span.col, path_ty));
                        }
                        return Ok(Type::Result(Box::new(Type::Void), Box::new(Type::Str)));
                    }
                    "env::args" => {
                        if !args.is_empty() {
                            return Err(format!("Tip Hatası [{}:{}]: 'env::args' argüman beklemez: env::args()", span.line, span.col));
                        }
                        return Ok(Type::Array(Box::new(Type::Str)));
                    }
                    "env::var" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'env::var' 1 argüman bekler: env::var(anahtar: str)", span.line, span.col));
                        }
                        let key_ty = self.check_expr(&args[0])?;
                        if key_ty != Type::Str {
                            return Err(format!("Tip Hatası [{}:{}]: 'env::var' anahtarı 'str' olmalıdır, bulunan: '{}'", span.line, span.col, key_ty));
                        }
                        return Ok(Type::Option(Box::new(Type::Str)));
                    }
                    "env::current_dir" | "current_dir" => {
                        if !args.is_empty() {
                            return Err(format!("Tip Hatası [{}:{}]: 'env::current_dir' argüman beklemez: env::current_dir()", span.line, span.col));
                        }
                        return Ok(Type::Result(Box::new(Type::Str), Box::new(Type::Str)));
                    }
                    "write_file" => {
                        if args.len() != 2 {
                            return Err(format!("Tip Hatası [{}:{}]: 'write_file' fonksiyonu 2 argüman bekler: write_file(yol: str, icerik: str)", span.line, span.col));
                        }
                        let path_ty = self.check_expr(&args[0])?;
                        let content_ty = self.check_expr(&args[1])?;
                        if path_ty != Type::Str {
                            return Err(format!("Tip Hatası [{}:{}]: 'write_file' ilk argüman olarak 'str' dosya yolu bekler, bulunan: '{}'", span.line, span.col, path_ty));
                        }
                        if content_ty != Type::Str {
                            return Err(format!("Tip Hatası [{}:{}]: 'write_file' ikinci argüman olarak 'str' içerik bekler, bulunan: '{}'", span.line, span.col, content_ty));
                        }
                        return Ok(Type::Bool);
                    }
                    "read_line" => {
                        if !args.is_empty() {
                            return Err(format!("Tip Hatası [{}:{}]: 'read_line' argüman beklemez: read_line()", span.line, span.col));
                        }
                        return Ok(Type::Str);
                    }
                    "to_int" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'to_int' fonksiyonu 1 argüman bekler", span.line, span.col));
                        }
                        let ty = self.check_expr(&args[0])?;
                        if ty == Type::Char || ty == Type::U8 || ty == Type::Int || ty == Type::Float {
                            return Ok(Type::Int);
                        } else {
                            return Err(format!("Tip Hatası [{}:{}]: 'to_int' yalnızca char, u8, int veya float kabul eder, bulunan: '{}'", span.line, span.col, ty));
                        }
                    }
                    "to_float" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'to_float' fonksiyonu 1 argüman bekler", span.line, span.col));
                        }
                        let ty = self.check_expr(&args[0])?;
                        if ty == Type::Int || ty == Type::Float || ty == Type::U8 {
                            return Ok(Type::Float);
                        } else {
                            return Err(format!("Tip Hatası [{}:{}]: 'to_float' yalnızca int, float veya u8 kabul eder, bulunan: '{}'", span.line, span.col, ty));
                        }
                    }
                    "to_char" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'to_char' fonksiyonu 1 argüman bekler", span.line, span.col));
                        }
                        let ty = self.check_expr(&args[0])?;
                        if ty == Type::Int || ty == Type::U8 || ty == Type::Char {
                            return Ok(Type::Char);
                        } else {
                            return Err(format!("Tip Hatası [{}:{}]: 'to_char' yalnızca int, u8 veya char kabul eder, bulunan: '{}'", span.line, span.col, ty));
                        }
                    }
                    "to_u8" | "to_byte" => {
                        if args.len() != 1 {
                            return Err(format!("Tip Hatası [{}:{}]: 'to_u8' fonksiyonu 1 argüman bekler", span.line, span.col));
                        }
                        let ty = self.check_expr(&args[0])?;
                        if ty == Type::Int || ty == Type::Char || ty == Type::U8 {
                            return Ok(Type::U8);
                        } else {
                            return Err(format!("Tip Hatası [{}:{}]: 'to_u8' yalnızca int, char veya u8 kabul eder, bulunan: '{}'", span.line, span.col, ty));
                        }
                    }
                    "trim" => {
                        if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'trim' 1 argüman bekler: trim(s: str)", span.line, span.col)); }
                        let ty = self.check_expr(&args[0])?;
                        if ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'trim' 'str' bekler", span.line, span.col)); }
                        return Ok(Type::Str);
                    }
                    "to_upper" | "to_uppercase" => {
                        if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'to_upper' 1 argüman bekler", span.line, span.col)); }
                        let ty = self.check_expr(&args[0])?;
                        if ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'to_upper' 'str' bekler", span.line, span.col)); }
                        return Ok(Type::Str);
                    }
                    "to_lower" | "to_lowercase" => {
                        if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'to_lower' 1 argüman bekler", span.line, span.col)); }
                        let ty = self.check_expr(&args[0])?;
                        if ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'to_lower' 'str' bekler", span.line, span.col)); }
                        return Ok(Type::Str);
                    }
                    "contains" => {
                        if args.len() != 2 { return Err(format!("Tip Hatası [{}:{}]: 'contains' 2 argüman bekler: contains(s: str, sub: str)", span.line, span.col)); }
                        let s_ty = self.check_expr(&args[0])?;
                        let sub_ty = self.check_expr(&args[1])?;
                        if s_ty != Type::Str || sub_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'contains' 'str' bekler", span.line, span.col)); }
                        return Ok(Type::Bool);
                    }
                    "split" => {
                        if args.len() != 2 { return Err(format!("Tip Hatası [{}:{}]: 'split' 2 argüman bekler: split(s: str, delim: str)", span.line, span.col)); }
                        let s_ty = self.check_expr(&args[0])?;
                        let delim_ty = self.check_expr(&args[1])?;
                        if s_ty != Type::Str || delim_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'split' 'str' bekler", span.line, span.col)); }
                        return Ok(Type::Array(Box::new(Type::Str)));
                    }
                    "replace" => {
                        if args.len() != 3 { return Err(format!("Tip Hatası [{}:{}]: 'replace' 3 argüman bekler: replace(s, eski, yeni)", span.line, span.col)); }
                        let s_ty = self.check_expr(&args[0])?;
                        let from_ty = self.check_expr(&args[1])?;
                        let to_ty = self.check_expr(&args[2])?;
                        if s_ty != Type::Str || from_ty != Type::Str || to_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'replace' 'str' bekler", span.line, span.col)); }
                        return Ok(Type::Str);
                    }
                    "char_at" => {
                        if args.len() != 2 { return Err(format!("Tip Hatası [{}:{}]: 'char_at' 2 argüman bekler: char_at(s: str, idx: int)", span.line, span.col)); }
                        let s_ty = self.check_expr(&args[0])?;
                        let idx_ty = self.check_expr(&args[1])?;
                        if s_ty != Type::Str || idx_ty != Type::Int { return Err(format!("Tip Hatası [{}:{}]: 'char_at' str ve int bekler", span.line, span.col)); }
                        return Ok(Type::Char);
                    }
                    "args" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'args()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Array(Box::new(Type::Str)));
                    }
                    "get_env" => {
                        if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'get_env' 1 argüman bekler: get_env(anahtar: str)", span.line, span.col)); }
                        let ty = self.check_expr(&args[0])?;
                        if ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'get_env' 'str' bekler", span.line, span.col)); }
                        return Ok(Type::Str);
                    }
                    "exit" => {
                        if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'exit' 1 argüman bekler: exit(kod: int)", span.line, span.col)); }
                        let ty = self.check_expr(&args[0])?;
                        if ty != Type::Int { return Err(format!("Tip Hatası [{}:{}]: 'exit' 'int' çıkış kodu bekler", span.line, span.col)); }
                        return Ok(Type::Void);
                    }
                    "map_new" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'map_new()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Named("Map".to_string()));
                    }
                    "map_set" => {
                        if args.len() != 3 { return Err(format!("Tip Hatası [{}:{}]: 'map_set' 3 argüman bekler: map_set(m, anahtar: str, deger)", span.line, span.col)); }
                        let m_ty = self.check_expr(&args[0])?;
                        let k_ty = self.check_expr(&args[1])?;
                        if m_ty != Type::Named("Map".to_string()) || k_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'map_set' Map ve str anahtar bekler", span.line, span.col)); }
                        self.check_expr(&args[2])?;
                        return Ok(Type::Void);
                    }
                    "map_get" => {
                        if args.len() != 2 { return Err(format!("Tip Hatası [{}:{}]: 'map_get' 2 argüman bekler: map_get(m, anahtar: str)", span.line, span.col)); }
                        let m_ty = self.check_expr(&args[0])?;
                        let k_ty = self.check_expr(&args[1])?;
                        if m_ty != Type::Named("Map".to_string()) || k_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'map_get' Map ve str anahtar bekler", span.line, span.col)); }
                        return Ok(Type::Str);
                    }
                    "map_has" => {
                        if args.len() != 2 { return Err(format!("Tip Hatası [{}:{}]: 'map_has' 2 argüman bekler: map_has(m, anahtar: str)", span.line, span.col)); }
                        let m_ty = self.check_expr(&args[0])?;
                        let k_ty = self.check_expr(&args[1])?;
                        if m_ty != Type::Named("Map".to_string()) || k_ty != Type::Str { return Err(format!("Tip Hatası [{}:{}]: 'map_has' Map ve str anahtar bekler", span.line, span.col)); }
                        return Ok(Type::Bool);
                    }
                    "map_len" => {
                        if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'map_len' 1 argüman bekler: map_len(m)", span.line, span.col)); }
                        let m_ty = self.check_expr(&args[0])?;
                        if m_ty != Type::Named("Map".to_string()) { return Err(format!("Tip Hatası [{}:{}]: 'map_len' Map bekler", span.line, span.col)); }
                        return Ok(Type::Int);
                    }
                    "map_keys" => {
                        if args.len() != 1 { return Err(format!("Tip Hatası [{}:{}]: 'map_keys' 1 argüman bekler: map_keys(m)", span.line, span.col)); }
                        let m_ty = self.check_expr(&args[0])?;
                        if m_ty != Type::Named("Map".to_string()) { return Err(format!("Tip Hatası [{}:{}]: 'map_keys' Map bekler", span.line, span.col)); }
                        return Ok(Type::Array(Box::new(Type::Str)));
                    }
                    "arena_reset" => {
                        if !args.is_empty() { return Err(format!("Tip Hatası [{}:{}]: 'arena_reset()' argüman beklemez", span.line, span.col)); }
                        return Ok(Type::Void);
                    }
                    _ => {}
                }

                // Check if calling a variable of Type::Fn (closure/lambda)
                if let Ok(sym) = self.lookup_var(callee, *span) {
                    if let Type::Fn { params, ret } = &sym.ty {
                        if params.len() != args.len() {
                            return Err(format!(
                                "Tip Hatası [{}:{}]: '{}' closure'ı {} argüman beklerken {} argüman verildi",
                                span.line, span.col, callee, params.len(), args.len()
                            ));
                        }
                        for (idx, (expected, arg_expr)) in params.iter().zip(args.iter()).enumerate() {
                            let arg_ty = self.check_expr(arg_expr)?;
                            if *expected != arg_ty {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: '{}' closure'ının {}. argümanı için '{}' beklenirken '{}' verildi",
                                    span.line, span.col, callee, idx + 1, expected, arg_ty
                                ));
                            }
                        }
                        return Ok((**ret).clone());
                    }
                }

                let func = match self.functions.get(callee) {
                    Some(f) => f.clone(),
                    None => return Err(format!("Tip Hatası [{}:{}]: Tanımsız fonksiyon çağrısı '{}'", span.line, span.col, callee)),
                };

                if self.extern_functions.contains(callee) && !self.is_unsafe {
                    return Err(format!(
                        "Güvenlik Hatası [{}:{}]: Harici C fonksiyonu '{}' çağrısı 'unsafe' blok içinde yapılmalıdır",
                        span.line, span.col, callee
                    ));
                }

                if func.params.len() != args.len() {
                    return Err(format!(
                        "Tip Hatası [{}:{}]: '{}' fonksiyonu {} argüman beklerken {} argüman verildi",
                        span.line, span.col, callee, func.params.len(), args.len()
                    ));
                }

                for (idx, (param, arg_expr)) in func.params.iter().zip(args.iter()).enumerate() {
                    let arg_ty = self.check_expr(arg_expr)?;
                    if !self.are_types_compatible(&param.ty, &arg_ty) {
                        return Err(format!(
                            "Tip Hatası [{}:{}]: '{}' fonksiyonunun {}. argümanı ('{}') için '{}' beklenirken '{}' verildi",
                            span.line, span.col, callee, idx + 1, param.name, param.ty, arg_ty
                        ));
                    }
                }

                Ok(func.return_type)
            }
            Expr::Path { base, member, span } => {
                let e_def = match self.enums.get(base) {
                    Some(e) => e,
                    None => return Err(format!("Tip Hatası [{}:{}]: Tanımsız enum '{}'", span.line, span.col, base)),
                };

                if !e_def.variants.contains(member) {
                    return Err(format!("Tip Hatası [{}:{}]: '{}' enum'ında '{}' adında bir varyant yok", span.line, span.col, base, member));
                }

                Ok(Type::Named(base.clone()))
            }
            Expr::If { condition, then_branch, else_branch, span } => {
                let cond_ty = self.check_expr(condition)?;
                if cond_ty != Type::Bool {
                    return Err(format!("Tip Hatası [{}:{}]: 'if' koşulu 'bool' tipinde olmalıdır, bulunan: '{}'", span.line, span.col, cond_ty));
                }

                let then_ty = self.check_block(then_branch)?;
                if let Some(eb) = else_branch {
                    let else_ty = self.check_block(eb)?;
                    if then_ty != else_ty {
                        return Err(format!(
                            "Tip Hatası [{}:{}]: 'if' ve 'else' dallarının dönüş tipleri uyuşmuyor: '{}' ve '{}'",
                            span.line, span.col, then_ty, else_ty
                        ));
                    }
                    Ok(then_ty)
                } else {
                    Ok(Type::Void)
                }
            }
            Expr::Block(block) => self.check_block(block),
            Expr::UnsafeBlock(block, _) => {
                let prev = self.is_unsafe;
                self.is_unsafe = true;
                let res = self.check_block(block);
                self.is_unsafe = prev;
                res
            }
            Expr::Borrow { expr, is_mut, span } => {
                let inner_ty = self.check_expr(expr)?;
                if *is_mut {
                    match &**expr {
                        Expr::Ident(name, id_span) => {
                            let sym = self.lookup_var(name, *id_span)?;
                            if !sym.is_mut {
                                return Err(format!(
                                    "Borrow Hatası [{}:{}]: Değişmez (immutable) '{}' değişkeni değiştirilebilir (&mut) ödünç alınamaz",
                                    span.line, span.col, name
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Type::Ref {
                    inner: Box::new(inner_ty),
                    is_mut: *is_mut,
                })
            }
            Expr::Deref { expr, span } => {
                let ty = self.check_expr(expr)?;
                match ty {
                    Type::Box(inner) | Type::Rc(inner) => Ok(*inner),
                    Type::Ref { inner, .. } => Ok(*inner),
                    Type::RawPtr { inner, .. } => {
                        if !self.is_unsafe {
                            return Err(format!(
                                "Güvenlik Hatası [{}:{}]: Ham gösterici (raw pointer) dereferansı 'unsafe' blok içinde yapılmalıdır",
                                span.line, span.col
                            ));
                        }
                        Ok(*inner)
                    }
                    _ => Err(format!(
                        "Tip Hatası [{}:{}]: '*' dereferans operatörü yalnızca Box, Rc, referans (&) veya ham gösterici (*const/*mut) ile kullanılabilir, bulunan: '{}'",
                        span.line, span.col, ty
                    )),
                }
            }
            Expr::DerefAssign { target, value, span } => {
                let tgt_ty = self.check_expr(target)?;
                let val_ty = self.check_expr(value)?;
                match tgt_ty {
                    Type::Box(inner) => {
                        if !self.are_types_compatible(&inner, &val_ty) {
                            return Err(format!("Tip Hatası [{}:{}]: Box dereferans atamasında tip uyuşmazlığı (beklenen: '{}', bulunan: '{}')", span.line, span.col, inner, val_ty));
                        }
                        Ok(val_ty)
                    }
                    Type::Ref { inner, is_mut } => {
                        if !is_mut {
                            return Err(format!("Tip Hatası [{}:{}]: Değiştirilemez (&) referans üzerinden dereferans ataması yapılamaz", span.line, span.col));
                        }
                        if !self.are_types_compatible(&inner, &val_ty) {
                            return Err(format!("Tip Hatası [{}:{}]: Referans dereferans atamasında tip uyuşmazlığı (beklenen: '{}', bulunan: '{}')", span.line, span.col, inner, val_ty));
                        }
                        Ok(val_ty)
                    }
                    Type::RawPtr { inner, is_mut } => {
                        if !is_mut {
                            return Err(format!("Tip Hatası [{}:{}]: '*const' ham gösterici üzerinden atama yapılamaz", span.line, span.col));
                        }
                        if !self.is_unsafe {
                            return Err(format!("Güvenlik Hatası [{}:{}]: Ham göstericiye atama 'unsafe' blok içinde yapılmalıdır", span.line, span.col));
                        }
                        if !self.are_types_compatible(&inner, &val_ty) {
                            return Err(format!("Tip Hatası [{}:{}]: Ham gösterici dereferans atamasında tip uyuşmazlığı (beklenen: '{}', bulunan: '{}')", span.line, span.col, inner, val_ty));
                        }
                        Ok(val_ty)
                    }
                    other => Err(format!("Tip Hatası [{}:{}]: '{}' tipi dereferans ataması (*x = val) desteklemiyor", span.line, span.col, other)),
                }
            }
            Expr::Try(expr, span) => {
                let ty = self.check_expr(expr)?;
                match ty {
                    Type::Option(inner) => {
                        if let Some(ret) = &self.current_fn_return {
                            if !matches!(ret, Type::Option(_)) {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: '?' operatörü Option üzerinde kullanıldığında fonksiyonun da 'Option' dönmesi gerekir, mevcut dönüş tipi: '{}'",
                                    span.line, span.col, ret
                                ));
                            }
                        }
                        Ok(*inner)
                    }
                    Type::Result(ok, err) => {
                        if let Some(ret) = &self.current_fn_return {
                            if let Type::Result(_, ret_err) = ret {
                                if *err != **ret_err && *err != Type::Void {
                                    return Err(format!(
                                        "Tip Hatası [{}:{}]: '?' hata tipi ('{}') fonksiyonun hata tipi ('{}') ile uyuşmuyor",
                                        span.line, span.col, err, ret_err
                                    ));
                                }
                            } else {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: '?' operatörü Result üzerinde kullanıldığında fonksiyonun da 'Result' dönmesi gerekir, mevcut dönüş tipi: '{}'",
                                    span.line, span.col, ret
                                ));
                            }
                        }
                        Ok(*ok)
                    }
                    _ => Err(format!(
                        "Tip Hatası [{}:{}]: '?' operatörü yalnızca Result veya Option tipleri üzerinde kullanılabilir, bulunan: '{}'",
                        span.line, span.col, ty
                    )),
                }
            }
            Expr::Match { subject, arms, span } => {
                let subj_ty = self.check_expr(subject)?;
                if arms.is_empty() {
                    return Err(format!("Tip Hatası [{}:{}]: 'match' ifadesinde en az bir dal (arm) bulunmalıdır", span.line, span.col));
                }

                let mut result_type: Option<Type> = None;

                for arm in arms {
                    self.enter_scope();
                    // Validate pattern against subject type
                    match &arm.pattern {
                        Pattern::Wildcard(..) => {}
                        Pattern::Int(_, p_span) => {
                            if subj_ty != Type::Int {
                                return Err(format!("Tip Hatası [{}:{}]: 'int' deseni '{}' tipindeki değere uygulanamaz", p_span.line, p_span.col, subj_ty));
                            }
                        }
                        Pattern::Float(_, p_span) => {
                            if subj_ty != Type::Float {
                                return Err(format!("Tip Hatası [{}:{}]: 'float' deseni '{}' tipindeki değere uygulanamaz", p_span.line, p_span.col, subj_ty));
                            }
                        }
                        Pattern::Bool(_, p_span) => {
                            if subj_ty != Type::Bool {
                                return Err(format!("Tip Hatası [{}:{}]: 'bool' deseni '{}' tipindeki değere uygulanamaz", p_span.line, p_span.col, subj_ty));
                            }
                        }
                        Pattern::Str(_, p_span) => {
                            if subj_ty != Type::Str {
                                return Err(format!("Tip Hatası [{}:{}]: 'str' deseni '{}' tipindeki değere uygulanamaz", p_span.line, p_span.col, subj_ty));
                            }
                        }
                        Pattern::Char(_, p_span) => {
                            if subj_ty != Type::Char {
                                return Err(format!("Tip Hatası [{}:{}]: 'char' deseni '{}' tipindeki değere uygulanamaz", p_span.line, p_span.col, subj_ty));
                            }
                        }
                        Pattern::U8(_, p_span) => {
                            if subj_ty != Type::U8 {
                                return Err(format!("Tip Hatası [{}:{}]: 'u8' deseni '{}' tipindeki değere uygulanamaz", p_span.line, p_span.col, subj_ty));
                            }
                        }
                        Pattern::EnumVariant { enum_name, variant, span: p_span } => {
                            if subj_ty != Type::Named(enum_name.clone()) {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: '{}::{}' deseni '{}' tipindeki hedefle uyuşmuyor",
                                    p_span.line, p_span.col, enum_name, variant, subj_ty
                                ));
                            }
                        }
                        Pattern::Some(inner, p_span) => {
                            if let Type::Option(inner_ty) = &subj_ty {
                                if let Pattern::Ident(name, i_span) = &**inner {
                                    self.declare_var(name.clone(), (**inner_ty).clone(), false, *i_span)?;
                                }
                            } else {
                                return Err(format!("Tip Hatası [{}:{}]: 'Some' deseni yalnızca Option tiplerine uygulanabilir, bulunan: '{}'", p_span.line, p_span.col, subj_ty));
                            }
                        }
                        Pattern::None(p_span) => {
                            if !matches!(&subj_ty, Type::Option(_)) {
                                return Err(format!("Tip Hatası [{}:{}]: 'None' deseni yalnızca Option tiplerine uygulanabilir, bulunan: '{}'", p_span.line, p_span.col, subj_ty));
                            }
                        }
                        Pattern::Ok(inner, p_span) => {
                            if let Type::Result(ok_ty, _) = &subj_ty {
                                if let Pattern::Ident(name, i_span) = &**inner {
                                    self.declare_var(name.clone(), (**ok_ty).clone(), false, *i_span)?;
                                }
                            } else {
                                return Err(format!("Tip Hatası [{}:{}]: 'Ok' deseni yalnızca Result tiplerine uygulanabilir, bulunan: '{}'", p_span.line, p_span.col, subj_ty));
                            }
                        }
                        Pattern::Err(inner, p_span) => {
                            if let Type::Result(_, err_ty) = &subj_ty {
                                if let Pattern::Ident(name, i_span) = &**inner {
                                    self.declare_var(name.clone(), (**err_ty).clone(), false, *i_span)?;
                                }
                            } else {
                                return Err(format!("Tip Hatası [{}:{}]: 'Err' deseni yalnızca Result tiplerine uygulanabilir, bulunan: '{}'", p_span.line, p_span.col, subj_ty));
                            }
                        }
                        Pattern::Ident(name, p_span) => {
                            self.declare_var(name.clone(), subj_ty.clone(), false, *p_span)?;
                        }
                        Pattern::Tuple(pats, p_span) => {
                            if let Type::Tuple(elem_tys) = &subj_ty {
                                if pats.len() != elem_tys.len() {
                                    return Err(format!(
                                        "Tip Hatası [{}:{}]: Demet deseni eleman sayısı uyuşmuyor: beklenen {}, bulunan {}",
                                        p_span.line, p_span.col, elem_tys.len(), pats.len()
                                    ));
                                }
                                for (p, elem_ty) in pats.iter().zip(elem_tys.iter()) {
                                    self.bind_pattern(p, elem_ty)?;
                                }
                            } else {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: Demet deseni yalnızca demet tiplerine uygulanabilir, bulunan: '{}'",
                                    p_span.line, p_span.col, subj_ty
                                ));
                            }
                        }
                    }

                    let arm_body_ty = self.check_expr(&arm.body)?;
                    self.exit_scope();

                    if let Some(ref res_ty) = result_type {
                        if *res_ty != arm_body_ty {
                            return Err(format!(
                                "Tip Hatası [{}:{}]: 'match' dallarının dönüş tipleri uyuşmuyor: '{}' ve '{}'",
                                span.line, span.col, res_ty, arm_body_ty
                            ));
                        }
                    } else {
                        result_type = Some(arm_body_ty);
                    }
                }

                Ok(result_type.unwrap_or(Type::Void))
            }
            Expr::Closure { params, return_type, body, span } => {
                self.enter_scope();
                for p in params {
                    self.declare_var(p.name.clone(), p.ty.clone(), p.is_mut, *span)?;
                }
                self.check_block(body)?;
                let inferred_ret = if let Some(ref trailing) = body.trailing_expr {
                    self.check_expr(trailing)?
                } else {
                    Type::Void
                };
                if let Some(expected) = return_type {
                    if *expected != inferred_ret {
                        self.exit_scope();
                        return Err(format!(
                            "Tip Hatası [{}:{}]: Closure belirtilen dönüş tipi '{}' ile gövde tipi '{}' uyuşmuyor",
                            span.line, span.col, expected, inferred_ret
                        ));
                    }
                }
                self.exit_scope();
                let ret = return_type.clone().unwrap_or(inferred_ret);
                Ok(Type::Fn {
                    params: params.iter().map(|p| p.ty.clone()).collect(),
                    ret: Box::new(ret),
                })
            }
        }
    }

    fn bind_pattern(&mut self, pat: &Pattern, ty: &Type) -> Result<(), String> {
        match pat {
            Pattern::Wildcard(..) => Ok(()),
            Pattern::Int(..) => {
                if *ty != Type::Int {
                    return Err(format!("Tip Hatası: 'int' deseni '{}' tipindeki değere uygulanamaz", ty));
                }
                Ok(())
            }
            Pattern::Float(..) => {
                if *ty != Type::Float {
                    return Err(format!("Tip Hatası: 'float' deseni '{}' tipindeki değere uygulanamaz", ty));
                }
                Ok(())
            }
            Pattern::Bool(..) => {
                if *ty != Type::Bool {
                    return Err(format!("Tip Hatası: 'bool' deseni '{}' tipindeki değere uygulanamaz", ty));
                }
                Ok(())
            }
            Pattern::Str(..) => {
                if *ty != Type::Str {
                    return Err(format!("Tip Hatası: 'str' deseni '{}' tipindeki değere uygulanamaz", ty));
                }
                Ok(())
            }
            Pattern::Char(..) => {
                if *ty != Type::Char {
                    return Err(format!("Tip Hatası: 'char' deseni '{}' tipindeki değere uygulanamaz", ty));
                }
                Ok(())
            }
            Pattern::U8(..) => {
                if *ty != Type::U8 {
                    return Err(format!("Tip Hatası: 'u8' deseni '{}' tipindeki değere uygulanamaz", ty));
                }
                Ok(())
            }
            Pattern::Ident(name, p_span) => {
                if name != "_" {
                    self.declare_var(name.clone(), ty.clone(), false, *p_span)?;
                }
                Ok(())
            }
            Pattern::Tuple(pats, p_span) => {
                if let Type::Tuple(elem_tys) = ty {
                    if pats.len() != elem_tys.len() {
                        return Err(format!("Tip Hatası [{}:{}]: Demet deseni eleman sayısı uyuşmuyor", p_span.line, p_span.col));
                    }
                    for (p, elem_t) in pats.iter().zip(elem_tys.iter()) {
                        self.bind_pattern(p, elem_t)?;
                    }
                    Ok(())
                } else {
                    Err(format!("Tip Hatası [{}:{}]: Demet deseni yalnızca demet tiplerine uygulanabilir", p_span.line, p_span.col))
                }
            }
            Pattern::Some(inner, p_span) => {
                if let Type::Option(in_t) = ty {
                    self.bind_pattern(inner, in_t)
                } else {
                    Err(format!("Tip Hatası [{}:{}]: Some deseni Option tipi bekler", p_span.line, p_span.col))
                }
            }
            Pattern::None(..) => Ok(()),
            Pattern::Ok(inner, p_span) => {
                if let Type::Result(ok_t, _) = ty {
                    self.bind_pattern(inner, ok_t)
                } else {
                    Err(format!("Tip Hatası [{}:{}]: Ok deseni Result tipi bekler", p_span.line, p_span.col))
                }
            }
            Pattern::Err(inner, p_span) => {
                if let Type::Result(_, err_t) = ty {
                    self.bind_pattern(inner, err_t)
                } else {
                    Err(format!("Tip Hatası [{}:{}]: Err deseni Result tipi bekler", p_span.line, p_span.col))
                }
            }
            Pattern::EnumVariant { .. } => Ok(()),
        }
    }
}
