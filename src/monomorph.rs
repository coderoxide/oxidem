use std::collections::{HashMap, HashSet};
use crate::ast::*;

pub fn mangle_type(ty: &Type) -> String {
    match ty {
        Type::Int => "int".to_string(),
        Type::Float => "float".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Str => "str".to_string(),
        Type::Char => "char".to_string(),
        Type::U8 => "u8".to_string(),
        Type::Void => "void".to_string(),
        Type::Array(inner) => format!("arr_{}", mangle_type(inner)),
        Type::Slice(inner) => format!("slice_{}", mangle_type(inner)),
        Type::StrSlice => "str_slice".to_string(),
        Type::Option(inner) => format!("Option__{}", mangle_type(inner)),
        Type::Result(ok, err) => format!("Result__{}_{}", mangle_type(ok), mangle_type(err)),
        Type::Box(inner) => format!("Box__{}", mangle_type(inner)),
        Type::Rc(inner) => format!("Rc__{}", mangle_type(inner)),
        Type::RefCell(inner) => format!("RefCell__{}", mangle_type(inner)),
        Type::JoinHandle(inner) => format!("JoinHandle__{}", mangle_type(inner)),
        Type::Sender(inner) => format!("Sender__{}", mangle_type(inner)),
        Type::Receiver(inner) => format!("Receiver__{}", mangle_type(inner)),
        Type::Mutex(inner) => format!("Mutex__{}", mangle_type(inner)),
        Type::TcpListener => "TcpListener".to_string(),
        Type::TcpStream => "TcpStream".to_string(),
        Type::Tuple(types) => {
            let inner = types.iter().map(mangle_type).collect::<Vec<_>>().join("_");
            format!("tuple_{}", inner)
        }
        Type::Named(name) => name.clone(),
        Type::Generic { name, type_args } => {
            let args_str = type_args.iter().map(mangle_type).collect::<Vec<_>>().join("_");
            format!("{}__{}", name, args_str)
        }
        Type::Ref { inner, is_mut } => {
            format!("ref_{}{}", if *is_mut { "mut_" } else { "" }, mangle_type(inner))
        }
        Type::RawPtr { inner, is_mut } => {
            format!("ptr_{}{}", if *is_mut { "mut_" } else { "" }, mangle_type(inner))
        }
        Type::Fn { params, ret } => {
            format!(
                "fn_{}_ret_{}",
                params.iter().map(mangle_type).collect::<Vec<_>>().join("_"),
                mangle_type(ret)
            )
        }
    }
}

pub fn normalize_type(ty: &Type) -> Type {
    match ty {
        Type::Generic { name, type_args } => {
            let norm_args: Vec<Type> = type_args.iter().map(normalize_type).collect();
            let mangled = format!("{}__{}", name, norm_args.iter().map(|a| mangle_type(&a)).collect::<Vec<_>>().join("_"));
            Type::Named(mangled)
        }
        Type::Array(inner) => Type::Array(Box::new(normalize_type(inner))),
        Type::Slice(inner) => Type::Slice(Box::new(normalize_type(inner))),
        Type::StrSlice => Type::StrSlice,
        Type::Option(inner) => Type::Option(Box::new(normalize_type(inner))),
        Type::Result(ok, err) => Type::Result(Box::new(normalize_type(ok)), Box::new(normalize_type(err))),
        Type::Box(inner) => Type::Box(Box::new(normalize_type(inner))),
        Type::Rc(inner) => Type::Rc(Box::new(normalize_type(inner))),
        Type::RefCell(inner) => Type::RefCell(Box::new(normalize_type(inner))),
        Type::JoinHandle(inner) => Type::JoinHandle(Box::new(normalize_type(inner))),
        Type::Sender(inner) => Type::Sender(Box::new(normalize_type(inner))),
        Type::Receiver(inner) => Type::Receiver(Box::new(normalize_type(inner))),
        Type::Mutex(inner) => Type::Mutex(Box::new(normalize_type(inner))),
        Type::Tuple(types) => Type::Tuple(types.iter().map(normalize_type).collect()),
        Type::Ref { inner, is_mut } => Type::Ref {
            inner: Box::new(normalize_type(inner)),
            is_mut: *is_mut,
        },
        Type::RawPtr { inner, is_mut } => Type::RawPtr {
            inner: Box::new(normalize_type(inner)),
            is_mut: *is_mut,
        },
        Type::Fn { params, ret } => Type::Fn {
            params: params.iter().map(normalize_type).collect(),
            ret: Box::new(normalize_type(ret)),
        },
        _ => ty.clone(),
    }
}

pub fn substitute_type(ty: &Type, mapping: &HashMap<String, Type>) -> Type {
    match ty {
        Type::Named(name) => {
            if let Some(sub) = mapping.get(name) {
                sub.clone()
            } else {
                Type::Named(name.clone())
            }
        }
        Type::Generic { name, type_args } => {
            let sub_args: Vec<Type> = type_args.iter().map(|a| substitute_type(a, mapping)).collect();
            Type::Generic {
                name: name.clone(),
                type_args: sub_args,
            }
        }
        Type::Array(inner) => Type::Array(Box::new(substitute_type(inner, mapping))),
        Type::Slice(inner) => Type::Slice(Box::new(substitute_type(inner, mapping))),
        Type::StrSlice => Type::StrSlice,
        Type::Option(inner) => Type::Option(Box::new(substitute_type(inner, mapping))),
        Type::Result(ok, err) => Type::Result(Box::new(substitute_type(ok, mapping)), Box::new(substitute_type(err, mapping))),
        Type::Box(inner) => Type::Box(Box::new(substitute_type(inner, mapping))),
        Type::Rc(inner) => Type::Rc(Box::new(substitute_type(inner, mapping))),
        Type::RefCell(inner) => Type::RefCell(Box::new(substitute_type(inner, mapping))),
        Type::JoinHandle(inner) => Type::JoinHandle(Box::new(substitute_type(inner, mapping))),
        Type::Sender(inner) => Type::Sender(Box::new(substitute_type(inner, mapping))),
        Type::Receiver(inner) => Type::Receiver(Box::new(substitute_type(inner, mapping))),
        Type::Mutex(inner) => Type::Mutex(Box::new(substitute_type(inner, mapping))),
        Type::Tuple(types) => Type::Tuple(types.iter().map(|t| substitute_type(t, mapping)).collect()),
        Type::Ref { inner, is_mut } => Type::Ref {
            inner: Box::new(substitute_type(inner, mapping)),
            is_mut: *is_mut,
        },
        Type::RawPtr { inner, is_mut } => Type::RawPtr {
            inner: Box::new(substitute_type(inner, mapping)),
            is_mut: *is_mut,
        },
        Type::Fn { params, ret } => Type::Fn {
            params: params.iter().map(|p| substitute_type(p, mapping)).collect(),
            ret: Box::new(substitute_type(ret, mapping)),
        },
        _ => ty.clone(),
    }
}

pub fn unify_types(
    param_ty: &Type,
    arg_ty: &Type,
    mapping: &mut HashMap<String, Type>,
    generic_params: &[GenericParam],
) -> Result<(), String> {
    if let Type::Named(name) = param_ty {
        if generic_params.iter().any(|g| g.name == *name) {
            if let Some(existing) = mapping.get(name) {
                if existing != arg_ty {
                    return Err(format!(
                        "Generic tip parametresi '{}' için çelişen tipler: '{}' ve '{}'",
                        name, existing, arg_ty
                    ));
                }
            } else {
                mapping.insert(name.clone(), arg_ty.clone());
            }
            return Ok(());
        }
    }

    match (param_ty, arg_ty) {
        (Type::Array(p_in), Type::Array(a_in)) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (Type::Slice(p_in), Type::Slice(a_in)) | (Type::Slice(p_in), Type::Array(a_in)) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (Type::StrSlice, Type::StrSlice) | (Type::StrSlice, Type::Str) => Ok(()),
        (Type::Option(p_in), Type::Option(a_in)) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (Type::Box(p_in), Type::Box(a_in)) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (Type::Rc(p_in), Type::Rc(a_in)) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (Type::RefCell(p_in), Type::RefCell(a_in)) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (Type::JoinHandle(p_in), Type::JoinHandle(a_in)) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (Type::Sender(p_in), Type::Sender(a_in)) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (Type::Receiver(p_in), Type::Receiver(a_in)) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (Type::Mutex(p_in), Type::Mutex(a_in)) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (Type::Result(p_ok, p_err), Type::Result(a_ok, a_err)) => {
            unify_types(p_ok, a_ok, mapping, generic_params)?;
            unify_types(p_err, a_err, mapping, generic_params)
        }
        (Type::Tuple(p_tys), Type::Tuple(a_tys)) => {
            if p_tys.len() != a_tys.len() {
                return Err(format!("Demet eleman sayısı uyumsuzluğu: '{}' ve '{}'", param_ty, arg_ty));
            }
            for (p, a) in p_tys.iter().zip(a_tys.iter()) {
                unify_types(p, a, mapping, generic_params)?;
            }
            Ok(())
        }
        (Type::Ref { inner: p_in, .. }, Type::Ref { inner: a_in, .. }) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (Type::RawPtr { inner: p_in, .. }, Type::RawPtr { inner: a_in, .. }) => {
            unify_types(p_in, a_in, mapping, generic_params)
        }
        (
            Type::Generic { name: p_name, type_args: p_args },
            Type::Generic { name: a_name, type_args: a_args },
        ) => {
            if p_name != a_name || p_args.len() != a_args.len() {
                return Err(format!("Tip uyumsuzluğu: '{}' ve '{}'", param_ty, arg_ty));
            }
            for (pa, aa) in p_args.iter().zip(a_args.iter()) {
                unify_types(pa, aa, mapping, generic_params)?;
            }
            Ok(())
        }
        (Type::Generic { name: p_name, .. }, Type::Named(a_name)) => {
            let prefix = format!("{}_", p_name);
            if a_name.starts_with(&prefix) {
                Ok(())
            } else {
                Err(format!("Tip uyumsuzluğu: '{}' ve '{}'", param_ty, arg_ty))
            }
        }
        (a, b) if a == b => Ok(()),
        _ => Err(format!("Tip uyumsuzluğu: '{}' beklenirken '{}' bulundu", param_ty, arg_ty)),
    }
}

pub struct Monomorphizer {
    generic_functions: HashMap<String, Function>,
    generic_structs: HashMap<String, StructDef>,
    generic_impls: HashMap<String, Vec<ImplBlock>>,
    traits: HashMap<String, TraitDef>,
    trait_impls: HashMap<String, HashSet<String>>,
    
    // Regular definitions for type inference
    concrete_functions: HashMap<String, (Vec<Type>, Type)>, // name -> (param_types, ret_type)
    concrete_structs: HashMap<String, Vec<(String, Type)>>,  // name -> fields
    concrete_methods: HashMap<String, HashMap<String, (Vec<Type>, Type)>>, // struct -> (method -> (param_tys, ret_ty))
    
    // Generated specializations
    specialized_functions: HashMap<String, Function>,
    specialized_structs: HashMap<String, StructDef>,
    specialized_impls: Vec<ImplBlock>,
}

impl Monomorphizer {
    pub fn new() -> Self {
        Self {
            generic_functions: HashMap::new(),
            generic_structs: HashMap::new(),
            generic_impls: HashMap::new(),
            traits: HashMap::new(),
            trait_impls: HashMap::new(),
            concrete_functions: HashMap::new(),
            concrete_structs: HashMap::new(),
            concrete_methods: HashMap::new(),
            specialized_functions: HashMap::new(),
            specialized_structs: HashMap::new(),
            specialized_impls: Vec::new(),
        }
    }

    pub fn monomorphize_program(program: Program) -> Result<Program, String> {
        let mut mono = Monomorphizer::new();
        mono.collect_definitions(&program);

        // Normalize all top-level types in concrete structs and functions
        let mut new_items = Vec::new();

        for item in program.items {
            match item {
                Item::Struct(mut s) => {
                    if s.generic_params.is_empty() {
                        for (_, f_ty) in &mut s.fields {
                            *f_ty = normalize_type(f_ty);
                        }
                        new_items.push(Item::Struct(s));
                    }
                }
                Item::Function(mut f) => {
                    if f.generic_params.is_empty() {
                        for p in &mut f.params {
                            p.ty = normalize_type(&p.ty);
                        }
                        f.return_type = normalize_type(&f.return_type);
                        let mut env = HashMap::new();
                        for p in &f.params {
                            env.insert(p.name.clone(), p.ty.clone());
                        }
                        mono.monomorphize_block(&mut f.body, &mut env)?;
                        new_items.push(Item::Function(f));
                    }
                }
                Item::Impl(mut imp) => {
                    // Synthesize default methods if this is a trait impl
                    if let Some(ref tr) = imp.trait_name {
                        if let Some(trait_def) = mono.traits.get(tr) {
                            for tm in &trait_def.methods {
                                if !imp.methods.iter().any(|m| m.name == tm.name) {
                                    if let Some(ref def_body) = tm.default_body {
                                        imp.methods.push(Function {
                                            name: tm.name.clone(),
                                            generic_params: Vec::new(),
                                            params: tm.params.clone(),
                                            return_type: tm.return_type.clone(),
                                            body: def_body.clone(),
                                            span: tm.span,
                                        });
                                    }
                                }
                            }
                        }
                    }

                    if imp.generic_params.is_empty() && !mono.generic_structs.contains_key(&imp.target) {
                        for m in &mut imp.methods {
                            for p in &mut m.params {
                                p.ty = normalize_type(&p.ty);
                            }
                            m.return_type = normalize_type(&m.return_type);
                            let mut env = HashMap::new();
                            for p in &m.params {
                                env.insert(p.name.clone(), p.ty.clone());
                            }
                            mono.monomorphize_block(&mut m.body, &mut env)?;
                        }
                        new_items.push(Item::Impl(imp));
                    }
                }
                Item::Trait(t) => {
                    new_items.push(Item::Trait(t));
                }
                other => new_items.push(other),
            }
        }

        // Add all specialized structs
        for (_, spec_s) in mono.specialized_structs {
            new_items.push(Item::Struct(spec_s));
        }

        // Add all specialized impls
        for spec_imp in mono.specialized_impls {
            new_items.push(Item::Impl(spec_imp));
        }

        // Add all specialized functions
        for (_, spec_f) in mono.specialized_functions {
            new_items.push(Item::Function(spec_f));
        }

        Ok(Program { items: new_items })
    }

    fn collect_definitions(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::Trait(t) => {
                    self.traits.insert(t.name.clone(), t.clone());
                }
                Item::Struct(s) => {
                    if !s.generic_params.is_empty() {
                        self.generic_structs.insert(s.name.clone(), s.clone());
                    } else {
                        self.concrete_structs.insert(s.name.clone(), s.fields.clone());
                    }
                }
                Item::Function(f) => {
                    if !f.generic_params.is_empty() {
                        self.generic_functions.insert(f.name.clone(), f.clone());
                    } else {
                        let param_tys = f.params.iter().map(|p| p.ty.clone()).collect();
                        self.concrete_functions.insert(f.name.clone(), (param_tys, f.return_type.clone()));
                    }
                }
                Item::Impl(imp) => {
                    if let Some(ref tr) = imp.trait_name {
                        self.trait_impls.entry(imp.target.clone()).or_insert_with(HashSet::new).insert(tr.clone());
                    }
                    for m in &imp.methods {
                        let param_tys = m.params.iter().map(|p| p.ty.clone()).collect();
                        self.concrete_methods.entry(imp.target.clone()).or_insert_with(HashMap::new).insert(m.name.clone(), (param_tys, m.return_type.clone()));
                    }
                    if !imp.generic_params.is_empty() || self.generic_structs.contains_key(&imp.target) {
                        self.generic_impls.entry(imp.target.clone()).or_insert_with(Vec::new).push(imp.clone());
                    }
                }
                Item::Extern(ext) => {
                    for f in &ext.functions {
                        let param_tys = f.params.iter().map(|p| p.ty.clone()).collect();
                        self.concrete_functions.insert(f.name.clone(), (param_tys, f.return_type.clone()));
                    }
                }
                _ => {}
            }
        }
    }

    fn monomorphize_block(&mut self, block: &mut Block, env: &mut HashMap<String, Type>) -> Result<(), String> {
        for stmt in &mut block.stmts {
            self.monomorphize_stmt(stmt, env)?;
        }
        if let Some(trailing) = &mut block.trailing_expr {
            self.monomorphize_expr(trailing, env)?;
        }
        Ok(())
    }

    fn monomorphize_stmt(&mut self, stmt: &mut Stmt, env: &mut HashMap<String, Type>) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, ty, init, .. } => {
                if let Some(t) = ty {
                    *t = normalize_type(t);
                    // If t is a generic struct instance e.g. Box__int, ensure struct is specialized
                    self.ensure_type_specialized(t)?;
                }
                if let Some(init_expr) = init {
                    self.monomorphize_expr(init_expr, env)?;
                    let inferred_ty = self.infer_expr_type(init_expr, env).unwrap_or(Type::Void);
                    let final_ty = ty.clone().unwrap_or(inferred_ty);
                    env.insert(name.clone(), final_ty);
                } else if let Some(t) = ty {
                    env.insert(name.clone(), t.clone());
                }
            }
            Stmt::LetTuple { elements, ty, init, .. } => {
                if let Some(t) = ty {
                    *t = normalize_type(t);
                    self.ensure_type_specialized(t)?;
                }
                let mut inferred_tuple = None;
                if let Some(init_expr) = init {
                    self.monomorphize_expr(init_expr, env)?;
                    if let Ok(inferred) = self.infer_expr_type(init_expr, env) {
                        inferred_tuple = Some(inferred);
                    }
                }
                if let Some(Type::Tuple(elem_tys)) = ty.clone().or(inferred_tuple) {
                    for (i, (name, _)) in elements.iter().enumerate() {
                        if name != "_" {
                            let elem_ty = elem_tys.get(i).cloned().unwrap_or(Type::Void);
                            env.insert(name.clone(), elem_ty);
                        }
                    }
                }
            }
            Stmt::While { condition, body, .. } => {
                self.monomorphize_expr(condition, env)?;
                let mut inner_env = env.clone();
                self.monomorphize_block(body, &mut inner_env)?;
            }
            Stmt::ForRange { var_name, start, end, body, .. } => {
                self.monomorphize_expr(start, env)?;
                self.monomorphize_expr(end, env)?;
                let mut inner_env = env.clone();
                inner_env.insert(var_name.clone(), Type::Int);
                self.monomorphize_block(body, &mut inner_env)?;
            }
            Stmt::ForIn { var_name, iterable, body, .. } => {
                self.monomorphize_expr(iterable, env)?;
                let elem_ty = match self.infer_expr_type(iterable, env) {
                    Ok(Type::Array(inner)) => *inner,
                    Ok(Type::Str) => Type::Char,
                    _ => Type::Int,
                };
                let mut inner_env = env.clone();
                inner_env.insert(var_name.clone(), elem_ty);
                self.monomorphize_block(body, &mut inner_env)?;
            }
            Stmt::Return(Some(expr), _) => {
                self.monomorphize_expr(expr, env)?;
            }
            Stmt::Return(None, _) => {}
            Stmt::Break(_) | Stmt::Continue(_) => {}
            Stmt::Const(c) => {
                c.ty = normalize_type(&c.ty);
                self.ensure_type_specialized(&c.ty)?;
                self.monomorphize_expr(&mut c.value, env)?;
                env.insert(c.name.clone(), c.ty.clone());
            }
            Stmt::Expr(expr) => {
                self.monomorphize_expr(expr, env)?;
            }
        }
        Ok(())
    }

    fn ensure_type_specialized(&mut self, ty: &Type) -> Result<(), String> {
        if let Type::Named(name) = ty {
            if let Some((base_name, args)) = self.parse_mangled_name(name) {
                if self.generic_structs.contains_key(&base_name) {
                    self.specialize_struct(&base_name, &args)?;
                }
            }
        } else if let Type::Array(inner) = ty {
            self.ensure_type_specialized(inner)?;
        } else if let Type::Ref { inner, .. } = ty {
            self.ensure_type_specialized(inner)?;
        }
        Ok(())
    }

    fn parse_mangled_name(&self, mangled: &str) -> Option<(String, Vec<Type>)> {
        if let Some(pos) = mangled.find("__") {
            let base = &mangled[..pos];
            let rest = &mangled[pos + 2..];
            let parts: Vec<&str> = rest.split('_').collect();
            let mut types = Vec::new();
            for p in parts {
                let ty = match p {
                    "int" => Type::Int,
                    "float" => Type::Float,
                    "bool" => Type::Bool,
                    "str" => Type::Str,
                    "char" => Type::Char,
                    "u8" => Type::U8,
                    "void" => Type::Void,
                    other => Type::Named(other.to_string()),
                };
                types.push(ty);
            }
            Some((base.to_string(), types))
        } else {
            None
        }
    }

    fn monomorphize_expr(&mut self, expr: &mut Expr, env: &mut HashMap<String, Type>) -> Result<(), String> {
        match expr {
            Expr::Call { callee, type_args, args, span } => {
                // First monomorphize all arguments
                for a in args.iter_mut() {
                    self.monomorphize_expr(a, env)?;
                }

                // If callee is a generic function:
                if self.generic_functions.contains_key(callee) {
                    let gen_fn = self.generic_functions[callee].clone();
                    let mut mapping = HashMap::new();

                    if !type_args.is_empty() {
                        if type_args.len() != gen_fn.generic_params.len() {
                            return Err(format!(
                                "Hata [{}:{}]: '{}' generic fonksiyonu {} tip parametresi bekliyor, {} verildi",
                                span.line, span.col, callee, gen_fn.generic_params.len(), type_args.len()
                            ));
                        }
                        for (p, a) in gen_fn.generic_params.iter().zip(type_args.iter()) {
                            mapping.insert(p.name.clone(), normalize_type(a));
                        }
                    } else {
                        // Infer from argument types!
                        for (param, arg_expr) in gen_fn.params.iter().zip(args.iter()) {
                            let arg_ty = self.infer_expr_type(arg_expr, env)?;
                            unify_types(&param.ty, &arg_ty, &mut mapping, &gen_fn.generic_params)?;
                        }
                    }

                    // Verify all generic params are resolved
                    let mut concrete_args = Vec::new();
                    for p in &gen_fn.generic_params {
                        match mapping.get(&p.name) {
                            Some(ty) => concrete_args.push(ty.clone()),
                            None => return Err(format!(
                                "Hata [{}:{}]: Generic parametre '{}' için tip çıkarımlanamadı. Lütfen 'fn::<...>' sözdizimi ile belirtin.",
                                span.line, span.col, p.name
                            )),
                        }
                    }

                    // Check trait bounds
                    for (p, concrete_ty) in gen_fn.generic_params.iter().zip(concrete_args.iter()) {
                        for bound in &p.bounds {
                            let implements = match concrete_ty {
                                Type::Named(s_name) => {
                                    self.trait_impls.get(s_name).map(|traits| traits.contains(bound)).unwrap_or(false)
                                }
                                _ => false,
                            };
                            if !implements {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: '{}' tipi '{}' trait sınırını karşılamıyor",
                                    span.line, span.col, concrete_ty, bound
                                ));
                            }
                        }
                    }

                    let mangled_name = format!("{}__{}", callee, concrete_args.iter().map(mangle_type).collect::<Vec<_>>().join("_"));

                    // Specialize function if not already specialized
                    self.specialize_function(callee, &concrete_args, &mangled_name)?;

                    *callee = mangled_name;
                    type_args.clear();
                }
            }
            Expr::StructInit { name, type_args, fields, span } => {
                for (_, f_expr) in fields.iter_mut() {
                    self.monomorphize_expr(f_expr, env)?;
                }

                if self.generic_structs.contains_key(name) {
                    let s_def = self.generic_structs[name].clone();
                    let mut mapping = HashMap::new();

                    if !type_args.is_empty() {
                        for (p, a) in s_def.generic_params.iter().zip(type_args.iter()) {
                            mapping.insert(p.name.clone(), normalize_type(a));
                        }
                    } else {
                        // Infer from fields
                        for (f_name, f_expr) in fields.iter() {
                            if let Some((_, f_def_ty)) = s_def.fields.iter().find(|(k, _)| k == f_name) {
                                let val_ty = self.infer_expr_type(f_expr, env)?;
                                unify_types(f_def_ty, &val_ty, &mut mapping, &s_def.generic_params)?;
                            }
                        }
                    }

                    let mut concrete_args = Vec::new();
                    for p in &s_def.generic_params {
                        match mapping.get(&p.name) {
                            Some(ty) => concrete_args.push(ty.clone()),
                            None => return Err(format!(
                                "Hata [{}:{}]: Struct '{}' generic parametresi '{}' için tip çıkarımlanamadı.",
                                span.line, span.col, name, p.name
                            )),
                        }
                    }

                    // Check trait bounds
                    for (p, concrete_ty) in s_def.generic_params.iter().zip(concrete_args.iter()) {
                        for bound in &p.bounds {
                            let implements = match concrete_ty {
                                Type::Named(s_name) => {
                                    self.trait_impls.get(s_name).map(|traits| traits.contains(bound)).unwrap_or(false)
                                }
                                _ => false,
                            };
                            if !implements {
                                return Err(format!(
                                    "Tip Hatası [{}:{}]: '{}' tipi '{}' trait sınırını karşılamıyor",
                                    span.line, span.col, concrete_ty, bound
                                ));
                            }
                        }
                    }

                    let mangled_name = format!("{}__{}", name, concrete_args.iter().map(mangle_type).collect::<Vec<_>>().join("_"));

                    self.specialize_struct(name, &concrete_args)?;

                    *name = mangled_name;
                    type_args.clear();
                }
            }
            Expr::MethodCall { target, method: _, type_args: _, args, span: _ } => {
                self.monomorphize_expr(target, env)?;
                for a in args.iter_mut() {
                    self.monomorphize_expr(a, env)?;
                }
            }
            Expr::Binary { left, right, .. } => {
                self.monomorphize_expr(left, env)?;
                self.monomorphize_expr(right, env)?;
            }
            Expr::Unary { expr, .. } => {
                self.monomorphize_expr(expr, env)?;
            }
            Expr::Assign { value, .. } => {
                self.monomorphize_expr(value, env)?;
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.monomorphize_expr(condition, env)?;
                let mut then_env = env.clone();
                self.monomorphize_block(then_branch, &mut then_env)?;
                if let Some(eb) = else_branch {
                    let mut else_env = env.clone();
                    self.monomorphize_block(eb, &mut else_env)?;
                }
            }
            Expr::Array(elements, _) => {
                for el in elements {
                    self.monomorphize_expr(el, env)?;
                }
            }
            Expr::Index { target, index, .. } => {
                self.monomorphize_expr(target, env)?;
                self.monomorphize_expr(index, env)?;
            }
            Expr::Slice { target, start, end, .. } => {
                self.monomorphize_expr(target, env)?;
                if let Some(s) = start {
                    self.monomorphize_expr(s, env)?;
                }
                if let Some(e) = end {
                    self.monomorphize_expr(e, env)?;
                }
            }
            Expr::IndexAssign { target, index, value, .. } => {
                self.monomorphize_expr(target, env)?;
                self.monomorphize_expr(index, env)?;
                self.monomorphize_expr(value, env)?;
            }
            Expr::FieldAccess { target, .. } => {
                self.monomorphize_expr(target, env)?;
            }
            Expr::FieldAssign { target, value, .. } => {
                self.monomorphize_expr(target, env)?;
                self.monomorphize_expr(value, env)?;
            }
            Expr::Borrow { expr, .. } => {
                self.monomorphize_expr(expr, env)?;
            }
            Expr::Deref { expr, .. } => {
                self.monomorphize_expr(expr, env)?;
            }
            Expr::Try(expr, ..) => {
                self.monomorphize_expr(expr, env)?;
            }
            Expr::UnsafeBlock(b, _) => {
                self.monomorphize_block(b, env)?;
            }
            Expr::Closure { params, return_type, body, .. } => {
                for p in params.iter_mut() {
                    p.ty = normalize_type(&p.ty);
                }
                if let Some(r) = return_type {
                    *r = normalize_type(r);
                }
                let mut closure_env = env.clone();
                for p in params {
                    closure_env.insert(p.name.clone(), p.ty.clone());
                }
                self.monomorphize_block(body, &mut closure_env)?;
            }
            Expr::Block(b) => {
                let mut inner_env = env.clone();
                self.monomorphize_block(b, &mut inner_env)?;
            }
            Expr::Match { subject, arms, .. } => {
                self.monomorphize_expr(subject, env)?;
                for arm in arms {
                    self.monomorphize_expr(&mut arm.body, env)?;
                }
            }
            Expr::Tuple(elements, _) => {
                for elem in elements {
                    self.monomorphize_expr(elem, env)?;
                }
            }
            Expr::TupleIndex { target, .. } => {
                self.monomorphize_expr(target, env)?;
            }
            Expr::TupleIndexAssign { target, value, .. } => {
                self.monomorphize_expr(target, env)?;
                self.monomorphize_expr(value, env)?;
            }
            Expr::DerefAssign { target, value, .. } => {
                self.monomorphize_expr(target, env)?;
                self.monomorphize_expr(value, env)?;
            }
            Expr::FormatString(parts, _) => {
                for part in parts {
                    if let FormatPart::Expr(e) = part {
                        self.monomorphize_expr(e, env)?;
                    }
                }
            }
            Expr::Int(..) | Expr::Float(..) | Expr::Bool(..) | Expr::Str(..) | Expr::Char(..) | Expr::U8(..) | Expr::Ident(..) | Expr::Path { .. } => {}
        }
        Ok(())
    }

    fn specialize_function(&mut self, fn_name: &str, concrete_args: &[Type], mangled_name: &str) -> Result<(), String> {
        if self.specialized_functions.contains_key(mangled_name) {
            return Ok(());
        }

        let gen_fn = match self.generic_functions.get(fn_name) {
            Some(f) => f.clone(),
            None => return Err(format!("Tanımsız generic fonksiyon '{}'", fn_name)),
        };

        let mut mapping = HashMap::new();
        for (p, a) in gen_fn.generic_params.iter().zip(concrete_args.iter()) {
            mapping.insert(p.name.clone(), a.clone());
        }

        let mut spec_params = Vec::new();
        for p in &gen_fn.params {
            spec_params.push(Param {
                name: p.name.clone(),
                ty: normalize_type(&substitute_type(&p.ty, &mapping)),
                is_mut: p.is_mut,
            });
        }

        let spec_ret = normalize_type(&substitute_type(&gen_fn.return_type, &mapping));

        let mut spec_body = gen_fn.body.clone();
        self.substitute_block(&mut spec_body, &mapping);

        // Pre-insert to avoid infinite recursion
        let placeholder = Function {
            name: mangled_name.to_string(),
            generic_params: Vec::new(),
            params: spec_params.clone(),
            return_type: spec_ret.clone(),
            body: Block { stmts: Vec::new(), trailing_expr: None, span: gen_fn.span },
            span: gen_fn.span,
        };
        self.specialized_functions.insert(mangled_name.to_string(), placeholder);

        let param_tys: Vec<Type> = spec_params.iter().map(|p| p.ty.clone()).collect();
        self.concrete_functions.insert(mangled_name.to_string(), (param_tys, spec_ret.clone()));

        // Now monomorphize the specialized body
        let mut fn_env = HashMap::new();
        for p in &spec_params {
            fn_env.insert(p.name.clone(), p.ty.clone());
        }
        self.monomorphize_block(&mut spec_body, &mut fn_env)?;

        let final_fn = Function {
            name: mangled_name.to_string(),
            generic_params: Vec::new(),
            params: spec_params,
            return_type: spec_ret,
            body: spec_body,
            span: gen_fn.span,
        };
        self.specialized_functions.insert(mangled_name.to_string(), final_fn);

        Ok(())
    }

    fn specialize_struct(&mut self, struct_name: &str, concrete_args: &[Type]) -> Result<String, String> {
        let mangled_name = format!("{}__{}", struct_name, concrete_args.iter().map(mangle_type).collect::<Vec<_>>().join("_"));

        if self.specialized_structs.contains_key(&mangled_name) {
            return Ok(mangled_name);
        }

        let gen_struct = match self.generic_structs.get(struct_name) {
            Some(s) => s.clone(),
            None => return Err(format!("Tanımsız generic struct '{}'", struct_name)),
        };

        let mut mapping = HashMap::new();
        for (p, a) in gen_struct.generic_params.iter().zip(concrete_args.iter()) {
            mapping.insert(p.name.clone(), a.clone());
        }

        let mut spec_fields = Vec::new();
        for (fname, fty) in &gen_struct.fields {
            let specialized_field_ty = normalize_type(&substitute_type(fty, &mapping));
            self.ensure_type_specialized(&specialized_field_ty)?;
            spec_fields.push((fname.clone(), specialized_field_ty));
        }

        let spec_struct = StructDef {
            name: mangled_name.clone(),
            generic_params: Vec::new(),
            fields: spec_fields.clone(),
            span: gen_struct.span,
        };

        self.specialized_structs.insert(mangled_name.clone(), spec_struct);
        self.concrete_structs.insert(mangled_name.clone(), spec_fields);

        // Also specialize any generic impl blocks for this struct
        if let Some(impl_list) = self.generic_impls.get(struct_name).cloned() {
            for imp in impl_list {
                let mut spec_methods = Vec::new();
                for m in imp.methods {
                    let mut m_params = Vec::new();
                    for p in &m.params {
                        let p_ty = if p.name == "self" {
                            Type::Named(mangled_name.clone())
                        } else {
                            normalize_type(&substitute_type(&p.ty, &mapping))
                        };
                        m_params.push(Param {
                            name: p.name.clone(),
                            ty: p_ty,
                            is_mut: p.is_mut,
                        });
                    }

                    let m_ret = normalize_type(&substitute_type(&m.return_type, &mapping));
                    let mut m_body = m.body.clone();
                    self.substitute_block(&mut m_body, &mapping);

                    let mut m_env = HashMap::new();
                    for p in &m_params {
                        m_env.insert(p.name.clone(), p.ty.clone());
                    }
                    self.monomorphize_block(&mut m_body, &mut m_env)?;

                    spec_methods.push(Function {
                        name: m.name.clone(),
                        generic_params: Vec::new(),
                        params: m_params,
                        return_type: m_ret,
                        body: m_body,
                        span: m.span,
                    });
                }

                self.specialized_impls.push(ImplBlock {
                    trait_name: imp.trait_name.clone(),
                    target: mangled_name.clone(),
                    generic_params: Vec::new(),
                    methods: spec_methods,
                    span: imp.span,
                });
            }
        }

        Ok(mangled_name)
    }

    fn substitute_block(&self, block: &mut Block, mapping: &HashMap<String, Type>) {
        for stmt in &mut block.stmts {
            self.substitute_stmt(stmt, mapping);
        }
        if let Some(trailing) = &mut block.trailing_expr {
            self.substitute_expr(trailing, mapping);
        }
    }

    fn substitute_stmt(&self, stmt: &mut Stmt, mapping: &HashMap<String, Type>) {
        match stmt {
            Stmt::Let { ty, init, .. } => {
                if let Some(t) = ty {
                    *t = normalize_type(&substitute_type(t, mapping));
                }
                if let Some(e) = init {
                    self.substitute_expr(e, mapping);
                }
            }
            Stmt::While { condition, body, .. } => {
                self.substitute_expr(condition, mapping);
                self.substitute_block(body, mapping);
            }
            Stmt::ForRange { start, end, body, .. } => {
                self.substitute_expr(start, mapping);
                self.substitute_expr(end, mapping);
                self.substitute_block(body, mapping);
            }
            Stmt::ForIn { iterable, body, .. } => {
                self.substitute_expr(iterable, mapping);
                self.substitute_block(body, mapping);
            }
            Stmt::Return(Some(e), _) => self.substitute_expr(e, mapping),
            Stmt::Expr(e) => self.substitute_expr(e, mapping),
            _ => {}
        }
    }

    fn substitute_expr(&self, expr: &mut Expr, mapping: &HashMap<String, Type>) {
        match expr {
            Expr::Call { type_args, args, .. } => {
                for ta in type_args {
                    *ta = normalize_type(&substitute_type(ta, mapping));
                }
                for a in args {
                    self.substitute_expr(a, mapping);
                }
            }
            Expr::StructInit { type_args, fields, .. } => {
                for ta in type_args {
                    *ta = normalize_type(&substitute_type(ta, mapping));
                }
                for (_, f_expr) in fields {
                    self.substitute_expr(f_expr, mapping);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.substitute_expr(left, mapping);
                self.substitute_expr(right, mapping);
            }
            Expr::Unary { expr, .. } => self.substitute_expr(expr, mapping),
            Expr::Assign { value, .. } => self.substitute_expr(value, mapping),
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.substitute_expr(condition, mapping);
                self.substitute_block(then_branch, mapping);
                if let Some(eb) = else_branch {
                    self.substitute_block(eb, mapping);
                }
            }
            Expr::Array(elements, _) => {
                for el in elements {
                    self.substitute_expr(el, mapping);
                }
            }
            Expr::Index { target, index, .. } => {
                self.substitute_expr(target, mapping);
                self.substitute_expr(index, mapping);
            }
            Expr::IndexAssign { target, index, value, .. } => {
                self.substitute_expr(target, mapping);
                self.substitute_expr(index, mapping);
                self.substitute_expr(value, mapping);
            }
            Expr::FieldAccess { target, .. } => self.substitute_expr(target, mapping),
            Expr::FieldAssign { target, value, .. } => {
                self.substitute_expr(target, mapping);
                self.substitute_expr(value, mapping);
            }
            Expr::MethodCall { target, type_args, args, .. } => {
                self.substitute_expr(target, mapping);
                for ta in type_args {
                    *ta = normalize_type(&substitute_type(ta, mapping));
                }
                for a in args {
                    self.substitute_expr(a, mapping);
                }
            }
            Expr::Borrow { expr, .. } => self.substitute_expr(expr, mapping),
            Expr::Deref { expr, .. } => self.substitute_expr(expr, mapping),
            Expr::Try(expr, ..) => self.substitute_expr(expr, mapping),
            Expr::Closure { params, return_type, body, .. } => {
                for p in params {
                    p.ty = normalize_type(&substitute_type(&p.ty, mapping));
                }
                if let Some(r) = return_type {
                    *r = normalize_type(&substitute_type(r, mapping));
                }
                self.substitute_block(body, mapping);
            }
            Expr::Block(b) => self.substitute_block(b, mapping),
            Expr::Match { subject, arms, .. } => {
                self.substitute_expr(subject, mapping);
                for arm in arms {
                    self.substitute_expr(&mut arm.body, mapping);
                }
            }
            _ => {}
        }
    }

    fn infer_expr_type(&self, expr: &Expr, env: &HashMap<String, Type>) -> Result<Type, String> {
        match expr {
            Expr::Int(..) => Ok(Type::Int),
            Expr::Float(..) => Ok(Type::Float),
            Expr::Bool(..) => Ok(Type::Bool),
            Expr::Str(..) => Ok(Type::Str),
            Expr::Char(..) => Ok(Type::Char),
            Expr::U8(..) => Ok(Type::U8),
            Expr::Ident(name, span) => {
                if let Some(ty) = env.get(name) {
                    Ok(ty.clone())
                } else {
                    Err(format!("Hata [{}:{}]: Tanımsız değişken '{}'", span.line, span.col, name))
                }
            }
            Expr::Binary { op, left, .. } => {
                let left_ty = self.infer_expr_type(left, env)?;
                if let Type::Named(ref s_name) = left_ty {
                    let m_name = match op {
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
                    if let Some(m) = m_name {
                        if let Some((_, ret)) = self.concrete_methods.get(s_name).and_then(|methods| methods.get(m)) {
                            return Ok(ret.clone());
                        }
                    }
                }
                match op {
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq | BinOp::And | BinOp::Or => Ok(Type::Bool),
                    _ => Ok(left_ty),
                }
            }
            Expr::Unary { op, expr, .. } => {
                let inner_ty = self.infer_expr_type(expr, env)?;
                if let Type::Named(ref s_name) = inner_ty {
                    let m_name = match op {
                        UnOp::Neg => "neg",
                        UnOp::Not => "not",
                    };
                    if let Some((_, ret)) = self.concrete_methods.get(s_name).and_then(|methods| methods.get(m_name)) {
                        return Ok(ret.clone());
                    }
                }
                match op {
                    UnOp::Not => Ok(Type::Bool),
                    UnOp::Neg => Ok(inner_ty),
                }
            }
            Expr::Array(elements, _) => {
                if let Some(first) = elements.first() {
                    let inner = self.infer_expr_type(first, env)?;
                    Ok(Type::Array(Box::new(inner)))
                } else {
                    Ok(Type::Array(Box::new(Type::Int)))
                }
            }
            Expr::Index { target, .. } => {
                match self.infer_expr_type(target, env)? {
                    Type::Array(inner) => Ok(*inner),
                    Type::Slice(inner) => Ok(*inner),
                    Type::Str => Ok(Type::Char),
                    Type::StrSlice => Ok(Type::Char),
                    Type::Named(ref s_name) => {
                        if let Some((_, ret)) = self.concrete_methods.get(s_name).and_then(|m| m.get("index")) {
                            Ok(ret.clone())
                        } else {
                            Ok(Type::Named(s_name.clone()))
                        }
                    }
                    other => Ok(other),
                }
            }
            Expr::Slice { target, .. } => {
                match self.infer_expr_type(target, env)? {
                    Type::Array(inner) => Ok(Type::Slice(inner)),
                    Type::Slice(inner) => Ok(Type::Slice(inner)),
                    Type::Str => Ok(Type::StrSlice),
                    Type::StrSlice => Ok(Type::StrSlice),
                    other => Ok(Type::Slice(Box::new(other))),
                }
            }
            Expr::Call { callee, args, .. } => {
                if let Some((_, ret)) = self.concrete_functions.get(callee) {
                    Ok(ret.clone())
                } else if let Some(gen_fn) = self.generic_functions.get(callee) {
                    Ok(gen_fn.return_type.clone())
                } else {
                    match callee.as_str() {
                        "Box::new" => {
                            if !args.is_empty() {
                                let inner = self.infer_expr_type(&args[0], env).unwrap_or(Type::Void);
                                return Ok(Type::Box(Box::new(inner)));
                            }
                            Ok(Type::Void)
                        }
                        "Rc::new" => {
                            if !args.is_empty() {
                                let inner = self.infer_expr_type(&args[0], env).unwrap_or(Type::Void);
                                return Ok(Type::Rc(Box::new(inner)));
                            }
                            Ok(Type::Void)
                        }
                        "Rc::strong_count" => Ok(Type::Int),
                        "RefCell::new" => {
                            if !args.is_empty() {
                                let inner = self.infer_expr_type(&args[0], env).unwrap_or(Type::Void);
                                return Ok(Type::RefCell(Box::new(inner)));
                            }
                            Ok(Type::Void)
                        }
                        "thread::spawn" => {
                            if !args.is_empty() {
                                if let Ok(Type::Fn { ret, .. }) = self.infer_expr_type(&args[0], env) {
                                    return Ok(Type::JoinHandle(ret));
                                }
                            }
                            Ok(Type::JoinHandle(Box::new(Type::Void)))
                        }
                        "thread::sleep" => Ok(Type::Void),
                        "channel" => Ok(Type::Tuple(vec![Type::Sender(Box::new(Type::Void)), Type::Receiver(Box::new(Type::Void))])),
                        "Mutex::new" => {
                            if !args.is_empty() {
                                let inner = self.infer_expr_type(&args[0], env).unwrap_or(Type::Void);
                                return Ok(Type::Mutex(Box::new(inner)));
                            }
                            Ok(Type::Void)
                        }
                        "TcpListener::bind" | "net::TcpListener::bind" => Ok(Type::Result(Box::new(Type::TcpListener), Box::new(Type::Str))),
                        "TcpStream::connect" | "net::TcpStream::connect" => Ok(Type::Result(Box::new(Type::TcpStream), Box::new(Type::Str))),
                        "http::request_method" | "http::request_path" | "http::request_body" | "http::response" | "json::stringify" => Ok(Type::Str),
                        "http::get" | "http::post" => Ok(Type::Result(Box::new(Type::Str), Box::new(Type::Str))),
                        "json::parse" => Ok(Type::Result(Box::new(Type::Named("Map".to_string())), Box::new(Type::Str))),
                        "json::get" => Ok(Type::Option(Box::new(Type::Str))),
                        "len" | "to_int" => Ok(Type::Int),
                        "to_float" => Ok(Type::Float),
                        "to_char" => Ok(Type::Char),
                        "to_u8" | "to_byte" => Ok(Type::U8),
                        "read_file" | "read_line" | "get_env" => Ok(Type::Str),
                        "write_file" => Ok(Type::Bool),
                        "args" => Ok(Type::Array(Box::new(Type::Str))),
                        _ => Ok(Type::Void),
                    }
                }
            }
            Expr::StructInit { name, .. } => Ok(Type::Named(name.clone())),
            Expr::FieldAccess { target, field, span } => {
                let mut target_ty = self.infer_expr_type(target, env)?;
                while let Type::Box(in_t) | Type::Rc(in_t) | Type::RefCell(in_t) = target_ty {
                    target_ty = *in_t;
                }
                if let Type::Named(s_name) = target_ty {
                    if let Some(fields) = self.concrete_structs.get(&s_name) {
                        if let Some((_, fty)) = fields.iter().find(|(k, _)| k == field) {
                            return Ok(fty.clone());
                        }
                    }
                }
                Err(format!("Hata [{}:{}]: '{}' alanına erişilemedi", span.line, span.col, field))
            }
            Expr::FieldAssign { .. } => Ok(Type::Void),
            Expr::Borrow { expr, is_mut, .. } => {
                let inner = self.infer_expr_type(expr, env)?;
                Ok(Type::Ref { inner: Box::new(inner), is_mut: *is_mut })
            }
            Expr::Deref { expr, .. } => {
                match self.infer_expr_type(expr, env)? {
                    Type::Box(inner) | Type::Rc(inner) | Type::RefCell(inner) => Ok(*inner),
                    Type::Ref { inner, .. } | Type::RawPtr { inner, .. } => Ok(*inner),
                    other => Ok(other),
                }
            }
            Expr::DerefAssign { .. } => Ok(Type::Void),
            Expr::Try(expr, ..) => {
                match self.infer_expr_type(expr, env)? {
                    Type::Option(inner) => Ok(*inner),
                    Type::Result(ok, _) => Ok(*ok),
                    other => Ok(other),
                }
            }
            Expr::Tuple(elements, _) => {
                let mut elem_tys = Vec::new();
                for e in elements {
                    elem_tys.push(self.infer_expr_type(e, env)?);
                }
                Ok(Type::Tuple(elem_tys))
            }
            Expr::TupleIndex { target, index, span } => {
                match self.infer_expr_type(target, env)? {
                    Type::Tuple(elem_tys) => {
                        if *index < elem_tys.len() {
                            Ok(elem_tys[*index].clone())
                        } else {
                            Err(format!("Hata [{}:{}]: Demet indeksi sınırların dışında", span.line, span.col))
                        }
                    }
                    other => Err(format!("Hata [{}:{}]: Demet indeksi yalnızca demetler için geçerlidir, bulunan: '{}'", span.line, span.col, other)),
                }
            }
            Expr::TupleIndexAssign { .. } => Ok(Type::Void),
            Expr::MethodCall { target, method, .. } => {
                if let Ok(target_ty) = self.infer_expr_type(target, env) {
                    match target_ty {
                        Type::JoinHandle(inner) if method == "join" => return Ok(Type::Result(inner, Box::new(Type::Str))),
                        Type::Sender(_) if method == "send" => return Ok(Type::Result(Box::new(Type::Void), Box::new(Type::Str))),
                        Type::Receiver(inner) if method == "recv" => return Ok(Type::Result(inner, Box::new(Type::Str))),
                        Type::Mutex(inner) if method == "lock" => return Ok(Type::Ref { inner, is_mut: true }),
                        Type::TcpListener if method == "accept" => return Ok(Type::Result(Box::new(Type::TcpStream), Box::new(Type::Str))),
                        Type::TcpStream if method == "write" => return Ok(Type::Result(Box::new(Type::Int), Box::new(Type::Str))),
                        Type::TcpStream if method == "read" => return Ok(Type::Result(Box::new(Type::Str), Box::new(Type::Str))),
                        Type::TcpStream if method == "close" => return Ok(Type::Void),
                        _ => {}
                    }
                }
                Ok(Type::Void)
            }
            Expr::FormatString(..) => Ok(Type::Str),
            _ => Ok(Type::Void),
        }
    }
}
