use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ast::*;
use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Char(char),
    U8(u8),
    Array(Rc<RefCell<Vec<Value>>>),
    Slice {
        target: Rc<RefCell<Vec<Value>>>,
        start: usize,
        len: usize,
    },
    StrSlice {
        target: Rc<String>,
        start: usize,
        len: usize,
    },
    Struct {
        name: String,
        fields: Rc<RefCell<HashMap<String, Value>>>,
    },
    EnumVariant {
        enum_name: String,
        variant: String,
    },
    Ref {
        val: Rc<RefCell<Value>>,
        is_mut: bool,
    },
    Map(Rc<RefCell<HashMap<String, Value>>>),
    Tuple(Rc<RefCell<Vec<Value>>>),
    Box(Rc<RefCell<Value>>),
    Rc(Rc<RefCell<Value>>, Rc<RefCell<usize>>),
    RefCell(Rc<RefCell<Value>>),
    Some(Box<Value>),
    None,
    Ok(Box<Value>),
    Err(Box<Value>),
    Closure {
        params: Vec<Param>,
        body: Block,
        env: Environment,
    },
    Void,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Char(c) => write!(f, "{}", c),
            Value::U8(b) => write!(f, "{}", b),
            Value::Ref { val, .. } => write!(f, "&{}", val.borrow()),
            Value::Closure { .. } => write!(f, "<closure>"),
            Value::Array(arr) => {
                let inner = arr.borrow();
                let elements = inner.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "[{}]", elements)
            }
            Value::Slice { target, start, len } => {
                let inner = target.borrow();
                let end = (*start + *len).min(inner.len());
                let slice_elems = &inner[*start..end];
                let elements = slice_elems.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "[{}]", elements)
            }
            Value::StrSlice { target, start, len } => {
                let end = (*start + *len).min(target.len());
                write!(f, "{}", &target[*start..end])
            }
            Value::Tuple(arr) => {
                let inner = arr.borrow();
                let elements = inner.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "({})", elements)
            }
            Value::Map(m) => {
                let inner = m.borrow();
                let mut entries = Vec::new();
                for (k, v) in inner.iter() {
                    entries.push(format!("\"{}\": {}", k, v));
                }
                write!(f, "Map {{ {} }}", entries.join(", "))
            }
            Value::Struct { name, fields } => {
                let inner = fields.borrow();
                let mut field_strs = Vec::new();
                for (k, v) in inner.iter() {
                    field_strs.push(format!("{}: {}", k, v));
                }
                write!(f, "{} {{ {} }}", name, field_strs.join(", "))
            }
            Value::EnumVariant { enum_name, variant } => {
                write!(f, "{}::{}", enum_name, variant)
            }
            Value::Box(inner) => write!(f, "Box({})", inner.borrow()),
            Value::Rc(inner, _) => write!(f, "Rc({})", inner.borrow()),
            Value::RefCell(inner) => write!(f, "RefCell({})", inner.borrow()),
            Value::Some(v) => write!(f, "Some({})", v),
            Value::None => write!(f, "None"),
            Value::Ok(v) => write!(f, "Ok({})", v),
            Value::Err(e) => write!(f, "Err({})", e),
            Value::Void => write!(f, "()"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Variable {
    value: Value,
    is_mut: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    scopes: Vec<HashMap<String, Variable>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn get_opt(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Some(var.value.clone());
            }
        }
        None
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn declare(&mut self, name: String, value: Value, is_mut: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, Variable { value, is_mut });
        }
    }

    pub fn is_mutable(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return var.is_mut;
            }
        }
        false
    }

    pub fn assign(&mut self, name: &str, value: Value, span: Span) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.get_mut(name) {
                if !var.is_mut {
                    return Err(format!(
                        "Hata [{}:{}]: Değiştirilemez (immutable) '{}' değişkenine yeniden atama yapılamaz! Değiştirmek istiyorsan 'let mut {}' şeklinde tanımlamalısın.",
                        span.line, span.col, name, name
                    ));
                }
                var.value = value;
                return Ok(());
            }
        }
        Err(format!(
            "Hata [{}:{}]: Tanımsız değişken '{}'",
            span.line, span.col, name
        ))
    }

    pub fn get(&self, name: &str, span: Span) -> Result<Value, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Ok(var.value.clone());
            }
        }
        Err(format!(
            "Hata [{}:{}]: Kapsamda bulunamayan değişken '{}'",
            span.line, span.col, name
        ))
    }
}

enum ControlFlow {
    Normal(Value),
    Return(Value),
    Break,
    Continue,
}

pub struct Interpreter {
    functions: HashMap<String, Function>,
    structs: HashMap<String, StructDef>,
    methods: HashMap<String, HashMap<String, Function>>,
    enums: HashMap<String, EnumDef>,
    constants: HashMap<String, Value>,
    env: Environment,
    early_return: Option<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            structs: HashMap::new(),
            methods: HashMap::new(),
            enums: HashMap::new(),
            constants: HashMap::new(),
            env: Environment::new(),
            early_return: None,
        }
    }

    pub fn load_program(&mut self, program: Program) {
        for item in program.items {
            match item {
                Item::Function(f) => {
                    self.functions.insert(f.name.clone(), f);
                }
                Item::Struct(s) => {
                    self.structs.insert(s.name.clone(), s);
                }
                Item::Impl(imp) => {
                    let entry = self.methods.entry(imp.target.clone()).or_insert_with(HashMap::new);
                    for m in imp.methods {
                        entry.insert(m.name.clone(), m);
                    }
                }
                Item::Enum(e) => {
                    self.enums.insert(e.name.clone(), e);
                }
                Item::Extern(ext) => {
                    for f in ext.functions {
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
                Item::Trait(..) => {}
                Item::Const(c) => {
                    let val = self.eval_expr(&c.value).unwrap_or(Value::Void);
                    self.constants.insert(c.name.clone(), val.clone());
                    self.env.declare(c.name, val, false);
                }
            }
        }
    }

    pub fn run_main(&mut self) -> Result<Value, String> {
        if !self.functions.contains_key("main") {
            return Err("Hata: Çalıştırılacak 'fn main()' fonksiyonu bulunamadı.".to_string());
        }

        self.call_function("main", vec![], Span::new(1, 1))
    }

    pub fn call_function(&mut self, name: &str, args: Vec<Value>, span: Span) -> Result<Value, String> {
        // Built-in functions
        match name {
            "Some" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'Some' 1 argüman bekler", span.line, span.col));
                }
                return Ok(Value::Some(Box::new(args.into_iter().next().unwrap())));
            }
            "Box::new" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'Box::new' 1 argüman bekler", span.line, span.col));
                }
                return Ok(Value::Box(Rc::new(RefCell::new(args.into_iter().next().unwrap()))));
            }
            "Rc::new" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'Rc::new' 1 argüman bekler", span.line, span.col));
                }
                return Ok(Value::Rc(Rc::new(RefCell::new(args.into_iter().next().unwrap())), Rc::new(RefCell::new(1))));
            }
            "Rc::strong_count" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'Rc::strong_count' 1 argüman bekler", span.line, span.col));
                }
                match &args[0] {
                    Value::Rc(_, count) => return Ok(Value::Int(*count.borrow() as i64)),
                    _ => return Err(format!("Hata [{}:{}]: 'Rc::strong_count' Rc<T> nesnesi bekler", span.line, span.col)),
                }
            }
            "RefCell::new" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'RefCell::new' 1 argüman bekler", span.line, span.col));
                }
                return Ok(Value::RefCell(Rc::new(RefCell::new(args.into_iter().next().unwrap()))));
            }
            "Ok" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'Ok' 1 argüman bekler", span.line, span.col));
                }
                return Ok(Value::Ok(Box::new(args.into_iter().next().unwrap())));
            }
            "Err" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'Err' 1 argüman bekler", span.line, span.col));
                }
                return Ok(Value::Err(Box::new(args.into_iter().next().unwrap())));
            }
            "format" => {
                if args.is_empty() {
                    return Ok(Value::Str(String::new()));
                }
                let mut template = args[0].to_string();
                for a in &args[1..] {
                    if let Some(pos) = template.find("{}") {
                        template.replace_range(pos..pos + 2, &a.to_string());
                    }
                }
                return Ok(Value::Str(template));
            }
            "println" => {
                let out = args
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("{}", out);
                return Ok(Value::Void);
            }
            "print" => {
                let out = args
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                print!("{}", out);
                use std::io::Write;
                std::io::stdout().flush().ok();
                return Ok(Value::Void);
            }
            "assert" => {
                if let Some(first) = args.first() {
                    match first {
                        Value::Bool(true) => return Ok(Value::Void),
                        _ => {
                            let msg = if args.len() > 1 {
                                args[1].to_string()
                            } else {
                                "Doğrulama başarısız oldu (assertion failed)".to_string()
                            };
                            return Err(format!("Assertion Hatası [{}:{}]: {}", span.line, span.col, msg));
                        }
                    }
                }
                return Err("assert en az 1 argüman bekler".to_string());
            }
            "len" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'len' fonksiyonu tam olarak 1 argüman bekler", span.line, span.col));
                }
                return match &args[0] {
                    Value::Array(arr) => Ok(Value::Int(arr.borrow().len() as i64)),
                    Value::Slice { len, .. } => Ok(Value::Int(*len as i64)),
                    Value::Str(s) => Ok(Value::Int(s.len() as i64)),
                    Value::StrSlice { len, .. } => Ok(Value::Int(*len as i64)),
                    _ => Err(format!("Hata [{}:{}]: 'len' yalnızca dizi, dilim veya metinler için kullanılabilir", span.line, span.col)),
                };
            }
            "push" => {
                if args.len() != 2 {
                    return Err(format!("Hata [{}:{}]: 'push' fonksiyonu 2 argüman bekler: push(dizi, eleman)", span.line, span.col));
                }
                match &args[0] {
                    Value::Array(arr) => {
                        arr.borrow_mut().push(args[1].clone());
                        return Ok(Value::Void);
                    }
                    _ => return Err(format!("Hata [{}:{}]: 'push' ilk argüman olarak dizi bekler", span.line, span.col)),
                }
            }
            "pop" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'pop' fonksiyonu 1 argüman bekler: pop(dizi)", span.line, span.col));
                }
                match &args[0] {
                    Value::Array(arr) => {
                        match arr.borrow_mut().pop() {
                            Some(val) => return Ok(val),
                            None => return Err(format!("Hata [{}:{}]: Boş diziden 'pop' yapılamaz", span.line, span.col)),
                        }
                    }
                    _ => return Err(format!("Hata [{}:{}]: 'pop' ilk argüman olarak dizi bekler", span.line, span.col)),
                }
            }
            "read_file" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'read_file' fonksiyonu 1 argüman bekler", span.line, span.col));
                }
                match &args[0] {
                    Value::Str(path) => {
                        match std::fs::read_to_string(path) {
                            Ok(content) => return Ok(Value::Str(content)),
                            Err(e) => return Err(format!("Dosya Hatası [{}:{}]: '{}' okunamadı: {}", span.line, span.col, path, e)),
                        }
                    }
                    _ => return Err(format!("Hata [{}:{}]: 'read_file' dosya yolunu str olarak bekler", span.line, span.col)),
                }
            }
            "write_file" => {
                if args.len() != 2 {
                    return Err(format!("Hata [{}:{}]: 'write_file' fonksiyonu 2 argüman bekler", span.line, span.col));
                }
                match (&args[0], &args[1]) {
                    (Value::Str(path), Value::Str(content)) => {
                        match std::fs::write(path, content) {
                            Ok(()) => return Ok(Value::Bool(true)),
                            Err(e) => return Err(format!("Dosya Hatası [{}:{}]: '{}' yazılamadı: {}", span.line, span.col, path, e)),
                        }
                    }
                    _ => return Err(format!("Hata [{}:{}]: 'write_file' str dosya yolu ve str içerik bekler", span.line, span.col)),
                }
            }
            "fs::read_to_string" | "read_to_string" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'fs::read_to_string' 1 argüman bekler", span.line, span.col));
                }
                match &args[0] {
                    Value::Str(path) => {
                        match std::fs::read_to_string(path) {
                            Ok(content) => return Ok(Value::Ok(Box::new(Value::Str(content)))),
                            Err(e) => return Ok(Value::Err(Box::new(Value::Str(e.to_string())))),
                        }
                    }
                    _ => return Err(format!("Hata [{}:{}]: 'fs::read_to_string' dosya yolunu str olarak bekler", span.line, span.col)),
                }
            }
            "fs::write" => {
                if args.len() != 2 {
                    return Err(format!("Hata [{}:{}]: 'fs::write' 2 argüman bekler", span.line, span.col));
                }
                match (&args[0], &args[1]) {
                    (Value::Str(path), Value::Str(content)) => {
                        match std::fs::write(path, content) {
                            Ok(()) => return Ok(Value::Ok(Box::new(Value::Void))),
                            Err(e) => return Ok(Value::Err(Box::new(Value::Str(e.to_string())))),
                        }
                    }
                    _ => return Err(format!("Hata [{}:{}]: 'fs::write' str yol ve str içerik bekler", span.line, span.col)),
                }
            }
            "fs::remove_file" | "remove_file" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'fs::remove_file' 1 argüman bekler", span.line, span.col));
                }
                match &args[0] {
                    Value::Str(path) => {
                        match std::fs::remove_file(path) {
                            Ok(()) => return Ok(Value::Ok(Box::new(Value::Void))),
                            Err(e) => return Ok(Value::Err(Box::new(Value::Str(e.to_string())))),
                        }
                    }
                    _ => return Err(format!("Hata [{}:{}]: 'fs::remove_file' str yol bekler", span.line, span.col)),
                }
            }
            "fs::exists" | "file_exists" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'fs::exists' 1 argüman bekler", span.line, span.col));
                }
                match &args[0] {
                    Value::Str(path) => return Ok(Value::Bool(std::path::Path::new(path).exists())),
                    _ => return Err(format!("Hata [{}:{}]: 'fs::exists' str yol bekler", span.line, span.col)),
                }
            }
            "fs::create_dir" | "create_dir" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'fs::create_dir' 1 argüman bekler", span.line, span.col));
                }
                match &args[0] {
                    Value::Str(path) => {
                        match std::fs::create_dir_all(path) {
                            Ok(()) => return Ok(Value::Ok(Box::new(Value::Void))),
                            Err(e) => return Ok(Value::Err(Box::new(Value::Str(e.to_string())))),
                        }
                    }
                    _ => return Err(format!("Hata [{}:{}]: 'fs::create_dir' str yol bekler", span.line, span.col)),
                }
            }
            "env::args" => {
                let args_vec: Vec<Value> = std::env::args().map(Value::Str).collect();
                return Ok(Value::Array(Rc::new(RefCell::new(args_vec))));
            }
            "env::var" => {
                if args.len() != 1 {
                    return Err(format!("Hata [{}:{}]: 'env::var' 1 argüman bekler", span.line, span.col));
                }
                match &args[0] {
                    Value::Str(key) => {
                        match std::env::var(key) {
                            Ok(val) => return Ok(Value::Some(Box::new(Value::Str(val)))),
                            Err(_) => return Ok(Value::None),
                        }
                    }
                    _ => return Err(format!("Hata [{}:{}]: 'env::var' str anahtar bekler", span.line, span.col)),
                }
            }
            "env::current_dir" | "current_dir" => {
                match std::env::current_dir() {
                    Ok(p) => return Ok(Value::Ok(Box::new(Value::Str(p.to_string_lossy().to_string())))),
                    Err(e) => return Ok(Value::Err(Box::new(Value::Str(e.to_string())))),
                }
            }
            "read_line" => {
                let mut buf = String::new();
                match std::io::stdin().read_line(&mut buf) {
                    Ok(_) => {
                        let trimmed = buf.trim_end_matches(&['\r', '\n'][..]).to_string();
                        return Ok(Value::Str(trimmed));
                    }
                    Err(e) => return Err(format!("Girdi Hatası [{}:{}]: Terminalden okuma yapılamadı: {}", span.line, span.col, e)),
                }
            }
            "to_int" => {
                if let Some(arg) = args.first() {
                    match arg {
                        Value::Char(c) => return Ok(Value::Int(*c as i64)),
                        Value::U8(b) => return Ok(Value::Int(*b as i64)),
                        Value::Int(i) => return Ok(Value::Int(*i)),
                        Value::Float(f) => return Ok(Value::Int(*f as i64)),
                        _ => return Err(format!("Hata [{}:{}]: 'to_int' için geçersiz argüman", span.line, span.col)),
                    }
                }
            }
            "to_float" => {
                if let Some(arg) = args.first() {
                    match arg {
                        Value::Int(i) => return Ok(Value::Float(*i as f64)),
                        Value::Float(f) => return Ok(Value::Float(*f)),
                        Value::U8(b) => return Ok(Value::Float(*b as f64)),
                        _ => return Err(format!("Hata [{}:{}]: 'to_float' için geçersiz argüman", span.line, span.col)),
                    }
                }
            }
            "to_char" => {
                if let Some(arg) = args.first() {
                    match arg {
                        Value::Int(i) => return Ok(Value::Char((*i as u8) as char)),
                        Value::U8(b) => return Ok(Value::Char(*b as char)),
                        Value::Char(c) => return Ok(Value::Char(*c)),
                        _ => return Err(format!("Hata [{}:{}]: 'to_char' için geçersiz argüman", span.line, span.col)),
                    }
                }
            }
            "to_u8" | "to_byte" => {
                if let Some(arg) = args.first() {
                    match arg {
                        Value::Int(i) => return Ok(Value::U8(*i as u8)),
                        Value::Char(c) => return Ok(Value::U8(*c as u8)),
                        Value::U8(b) => return Ok(Value::U8(*b)),
                        _ => return Err(format!("Hata [{}:{}]: 'to_u8' için geçersiz argüman", span.line, span.col)),
                    }
                }
            }
            "sqrt" => {
                if let Some(Value::Float(f)) = args.first() {
                    return Ok(Value::Float(f.sqrt()));
                }
            }
            "pow" => {
                if args.len() == 2 {
                    if let (Value::Float(b), Value::Float(e)) = (&args[0], &args[1]) {
                        return Ok(Value::Float(b.powf(*e)));
                    }
                }
            }
            "abs" => {
                if let Some(Value::Int(i)) = args.first() {
                    return Ok(Value::Int(i.abs()));
                } else if let Some(Value::Float(f)) = args.first() {
                    return Ok(Value::Float(f.abs()));
                }
            }
            "sin" => {
                if let Some(Value::Float(f)) = args.first() {
                    return Ok(Value::Float(f.sin()));
                }
            }
            "cos" => {
                if let Some(Value::Float(f)) = args.first() {
                    return Ok(Value::Float(f.cos()));
                }
            }
            "tan" => {
                if let Some(Value::Float(f)) = args.first() {
                    return Ok(Value::Float(f.tan()));
                }
            }
            "floor" => {
                if let Some(Value::Float(f)) = args.first() {
                    return Ok(Value::Float(f.floor()));
                }
            }
            "ceil" => {
                if let Some(Value::Float(f)) = args.first() {
                    return Ok(Value::Float(f.ceil()));
                }
            }
            "round" => {
                if let Some(Value::Float(f)) = args.first() {
                    return Ok(Value::Float(f.round()));
                }
            }
            "trim" => {
                if let Some(arg) = args.first() {
                    match arg {
                        Value::Str(s) => return Ok(Value::Str(s.trim().to_string())),
                        _ => return Err(format!("Hata [{}:{}]: 'trim' str bekler", span.line, span.col)),
                    }
                }
            }
            "to_upper" | "to_uppercase" => {
                if let Some(arg) = args.first() {
                    match arg {
                        Value::Str(s) => return Ok(Value::Str(s.to_uppercase())),
                        _ => return Err(format!("Hata [{}:{}]: 'to_upper' str bekler", span.line, span.col)),
                    }
                }
            }
            "to_lower" | "to_lowercase" => {
                if let Some(arg) = args.first() {
                    match arg {
                        Value::Str(s) => return Ok(Value::Str(s.to_lowercase())),
                        _ => return Err(format!("Hata [{}:{}]: 'to_lower' str bekler", span.line, span.col)),
                    }
                }
            }
            "contains" => {
                if args.len() == 2 {
                    if let (Value::Str(s), Value::Str(sub)) = (&args[0], &args[1]) {
                        return Ok(Value::Bool(s.contains(sub)));
                    }
                }
                return Err(format!("Hata [{}:{}]: 'contains' (str, str) bekler", span.line, span.col));
            }
            "split" => {
                if args.len() == 2 {
                    if let (Value::Str(s), Value::Str(sep)) = (&args[0], &args[1]) {
                        let parts: Vec<Value> = s.split(sep).map(|p| Value::Str(p.to_string())).collect();
                        return Ok(Value::Array(Rc::new(RefCell::new(parts))));
                    }
                }
                return Err(format!("Hata [{}:{}]: 'split' (str, str) bekler", span.line, span.col));
            }
            "replace" => {
                if args.len() == 3 {
                    if let (Value::Str(s), Value::Str(from), Value::Str(to)) = (&args[0], &args[1], &args[2]) {
                        return Ok(Value::Str(s.replace(from, to)));
                    }
                }
                return Err(format!("Hata [{}:{}]: 'replace' (str, str, str) bekler", span.line, span.col));
            }
            "char_at" => {
                if args.len() == 2 {
                    if let (Value::Str(s), Value::Int(idx)) = (&args[0], &args[1]) {
                        if *idx < 0 {
                            return Err(format!("Hata [{}:{}]: Negatif indeks geçersiz", span.line, span.col));
                        }
                        return match s.chars().nth(*idx as usize) {
                            Some(c) => Ok(Value::Char(c)),
                            None => Err(format!("Hata [{}:{}]: Metin indeksi sınır dışı", span.line, span.col)),
                        };
                    }
                }
                return Err(format!("Hata [{}:{}]: 'char_at' (str, int) bekler", span.line, span.col));
            }
            "args" => {
                let cli_args: Vec<Value> = std::env::args().map(Value::Str).collect();
                return Ok(Value::Array(Rc::new(RefCell::new(cli_args))));
            }
            "get_env" => {
                if let Some(Value::Str(key)) = args.first() {
                    return Ok(Value::Str(std::env::var(key).unwrap_or_default()));
                }
                return Err(format!("Hata [{}:{}]: 'get_env' str anahtar bekler", span.line, span.col));
            }
            "exit" => {
                if let Some(Value::Int(code)) = args.first() {
                    std::process::exit(*code as i32);
                }
                std::process::exit(0);
            }
            "map_new" => {
                return Ok(Value::Map(Rc::new(RefCell::new(HashMap::new()))));
            }
            "map_set" => {
                if args.len() == 3 {
                    if let Value::Map(m) = &args[0] {
                        let k = match &args[1] {
                            Value::Str(s) => s.clone(),
                            other => other.to_string(),
                        };
                        m.borrow_mut().insert(k, args[2].clone());
                        return Ok(Value::Void);
                    }
                }
                return Err(format!("Hata [{}:{}]: 'map_set' (map, anahtar, deger) bekler", span.line, span.col));
            }
            "map_get" => {
                if args.len() == 2 {
                    if let Value::Map(m) = &args[0] {
                        let k = match &args[1] {
                            Value::Str(s) => s.clone(),
                            other => other.to_string(),
                        };
                        return Ok(m.borrow().get(&k).cloned().unwrap_or(Value::Str(String::new())));
                    }
                }
                return Err(format!("Hata [{}:{}]: 'map_get' (map, anahtar) bekler", span.line, span.col));
            }
            "map_has" => {
                if args.len() == 2 {
                    if let Value::Map(m) = &args[0] {
                        let k = match &args[1] {
                            Value::Str(s) => s.clone(),
                            other => other.to_string(),
                        };
                        return Ok(Value::Bool(m.borrow().contains_key(&k)));
                    }
                }
                return Err(format!("Hata [{}:{}]: 'map_has' (map, anahtar) bekler", span.line, span.col));
            }
            "map_len" => {
                if let Some(Value::Map(m)) = args.first() {
                    return Ok(Value::Int(m.borrow().len() as i64));
                }
                return Err(format!("Hata [{}:{}]: 'map_len' Map bekler", span.line, span.col));
            }
            "map_keys" => {
                if let Some(Value::Map(m)) = args.first() {
                    let keys: Vec<Value> = m.borrow().keys().cloned().map(Value::Str).collect();
                    return Ok(Value::Array(Rc::new(RefCell::new(keys))));
                }
                return Err(format!("Hata [{}:{}]: 'map_keys' Map bekler", span.line, span.col));
            }
            "arena_reset" => {
                return Ok(Value::Void);
            }
            _ => {}
        }

        let func = match self.functions.get(name) {
            Some(f) => f.clone(),
            None => {
                return Err(format!(
                    "Hata [{}:{}]: Tanımsız fonksiyon çağrısı '{}'",
                    span.line, span.col, name
                ));
            }
        };

        if func.params.len() != args.len() {
            return Err(format!(
                "Hata [{}:{}]: '{}' fonksiyonu {} argüman beklerken {} argüman verildi",
                span.line, span.col, name, func.params.len(), args.len()
            ));
        }

        // Save current scopes and setup new scope for function execution
        let prev_env = std::mem::replace(&mut self.env, Environment::new());

        for (param, arg_val) in func.params.iter().zip(args.into_iter()) {
            let passed_val = match (&param.ty, arg_val) {
                (Type::Slice(..), Value::Array(arr)) => {
                    let total_len = arr.borrow().len();
                    Value::Slice {
                        target: arr,
                        start: 0,
                        len: total_len,
                    }
                }
                (Type::StrSlice, Value::Str(s)) => {
                    let total_len = s.len();
                    Value::StrSlice {
                        target: Rc::new(s),
                        start: 0,
                        len: total_len,
                    }
                }
                (_, val) => val,
            };
            self.env.declare(param.name.clone(), passed_val, param.is_mut);
        }

        let result = match self.eval_block(&func.body)? {
            ControlFlow::Normal(v) => v,
            ControlFlow::Return(v) => v,
            ControlFlow::Break | ControlFlow::Continue => {
                return Err(format!("Hata [{}:{}]: Döngü dışında 'break' veya 'continue' kullanılamaz", span.line, span.col));
            }
        };

        // Restore environment
        self.env = prev_env;

        Ok(result)
    }

    pub fn call_struct_method(&mut self, target_val: &Value, method: &str, evaluated_args: Vec<Value>, span: Span) -> Result<Value, String> {
        let struct_name = match target_val {
            Value::Struct { name, .. } => name.clone(),
            _ => return Err(format!("Hata [{}:{}]: Metot çağrısı struct üzerinde yapılmalıdır", span.line, span.col)),
        };

        let method_fn = match self.methods.get(&struct_name).and_then(|m| m.get(method)) {
            Some(f) => f.clone(),
            None => return Err(format!("Hata [{}:{}]: '{}' struct'ı için '{}' metodu bulunamadı", span.line, span.col, struct_name, method)),
        };

        let mut all_args = Vec::new();
        if let Some(first_param) = method_fn.params.first() {
            if first_param.name == "self" {
                all_args.push(target_val.clone());
            }
        }
        all_args.extend(evaluated_args);

        if method_fn.params.len() != all_args.len() {
            return Err(format!(
                "Hata [{}:{}]: '{}::{}' metodu {} argüman beklerken {} argüman verildi",
                span.line, span.col, struct_name, method, method_fn.params.len(), all_args.len()
            ));
        }

        let prev_env = std::mem::replace(&mut self.env, Environment::new());
        for (param, arg_val) in method_fn.params.iter().zip(all_args.into_iter()) {
            self.env.declare(param.name.clone(), arg_val, param.is_mut);
        }

        let result = match self.eval_block(&method_fn.body)? {
            ControlFlow::Normal(v) => v,
            ControlFlow::Return(v) => v,
            ControlFlow::Break | ControlFlow::Continue => {
                return Err(format!("Hata [{}:{}]: Döngü dışında 'break' veya 'continue' kullanılamaz", span.line, span.col));
            }
        };

        self.env = prev_env;
        Ok(result)
    }

    fn eval_block(&mut self, block: &Block) -> Result<ControlFlow, String> {
        self.env.enter_scope();

        for stmt in &block.stmts {
            match self.eval_stmt(stmt)? {
                ControlFlow::Normal(_) => {}
                cf @ (ControlFlow::Return(_) | ControlFlow::Break | ControlFlow::Continue) => {
                    self.env.exit_scope();
                    return Ok(cf);
                }
            }
            if let Some(early) = self.early_return.take() {
                self.env.exit_scope();
                return Ok(ControlFlow::Return(early));
            }
        }

        let res = if let Some(trailing) = &block.trailing_expr {
            let v = self.eval_expr(trailing)?;
            if let Some(early) = self.early_return.take() {
                self.env.exit_scope();
                return Ok(ControlFlow::Return(early));
            }
            ControlFlow::Normal(v)
        } else {
            ControlFlow::Normal(Value::Void)
        };

        self.env.exit_scope();
        Ok(res)
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<ControlFlow, String> {
        match stmt {
            Stmt::Let { name, is_mut, ty, init, .. } => {
                let mut val = if let Some(expr) = init {
                    let v = self.eval_expr(expr)?;
                    if let Some(early) = self.early_return.take() {
                        return Ok(ControlFlow::Return(early));
                    }
                    v
                } else {
                    Value::Void
                };
                if let (Some(Type::U8), Value::Int(i)) = (ty, &val) {
                    val = Value::U8(*i as u8);
                }
                if let (Some(Type::Slice(..)), Value::Array(arr)) = (ty, &val) {
                    let total_len = arr.borrow().len();
                    val = Value::Slice {
                        target: arr.clone(),
                        start: 0,
                        len: total_len,
                    };
                }
                if let (Some(Type::StrSlice), Value::Str(s)) = (ty, &val) {
                    let total_len = s.len();
                    val = Value::StrSlice {
                        target: Rc::new(s.clone()),
                        start: 0,
                        len: total_len,
                    };
                }
                self.env.declare(name.clone(), val, *is_mut);
                Ok(ControlFlow::Normal(Value::Void))
            }
            Stmt::Const(c) => {
                let val = self.eval_expr(&c.value)?;
                if let Some(early) = self.early_return.take() {
                    return Ok(ControlFlow::Return(early));
                }
                self.constants.insert(c.name.clone(), val.clone());
                self.env.declare(c.name.clone(), val, false);
                Ok(ControlFlow::Normal(Value::Void))
            }
            Stmt::LetTuple { elements, init, span, .. } => {
                let init_val = if let Some(expr) = init {
                    let v = self.eval_expr(expr)?;
                    if let Some(early) = self.early_return.take() {
                        return Ok(ControlFlow::Return(early));
                    }
                    v
                } else {
                    Value::Void
                };
                match init_val {
                    Value::Tuple(tuple_elems) => {
                        let borrowed = tuple_elems.borrow();
                        for (i, (name, is_mut)) in elements.iter().enumerate() {
                            if name != "_" {
                                let v = borrowed.get(i).cloned().unwrap_or(Value::Void);
                                self.env.declare(name.clone(), v, *is_mut);
                            }
                        }
                        Ok(ControlFlow::Normal(Value::Void))
                    }
                    _ => Err(format!(
                        "Çalışma Zamanı Hatası [{}:{}]: Demet ayrıştırması demet olmayan bir değer üzerinde yapılamaz",
                        span.line, span.col
                    )),
                }
            }
            Stmt::While { condition, body, .. } => {
                loop {
                    let cond_val = self.eval_expr(condition)?;
                    if !Self::is_truthy(&cond_val) {
                        break;
                    }
                    match self.eval_block(body)? {
                        ControlFlow::Normal(_) | ControlFlow::Continue => {}
                        ControlFlow::Break => break,
                        ret @ ControlFlow::Return(_) => return Ok(ret),
                    }
                }
                Ok(ControlFlow::Normal(Value::Void))
            }
            Stmt::ForRange { var_name, start, end, body, span } => {
                let start_val = match self.eval_expr(start)? {
                    Value::Int(i) => i,
                    _ => return Err(format!("Hata [{}:{}]: Aralık başlangıcı 'int' olmalıdır", span.line, span.col)),
                };
                let end_val = match self.eval_expr(end)? {
                    Value::Int(i) => i,
                    _ => return Err(format!("Hata [{}:{}]: Aralık bitişi 'int' olmalıdır", span.line, span.col)),
                };

                for i in start_val..end_val {
                    self.env.enter_scope();
                    self.env.declare(var_name.clone(), Value::Int(i), false);
                    let flow = self.eval_block(body);
                    self.env.exit_scope();
                    match flow? {
                        ControlFlow::Normal(_) | ControlFlow::Continue => {}
                        ControlFlow::Break => break,
                        ret @ ControlFlow::Return(_) => return Ok(ret),
                    }
                }
                Ok(ControlFlow::Normal(Value::Void))
            }
            Stmt::ForIn { var_name, iterable, body, span } => {
                let iter_val = self.eval_expr(iterable)?;
                let items: Vec<Value> = match iter_val {
                    Value::Array(arr) => arr.borrow().clone(),
                    Value::Slice { target, start, len } => {
                        let inner = target.borrow();
                        inner[start..start + len].to_vec()
                    }
                    Value::Str(s) => s.chars().map(|c| Value::Str(c.to_string())).collect(),
                    Value::StrSlice { target, start, len } => {
                        target[start..start + len].chars().map(|c| Value::Str(c.to_string())).collect()
                    }
                    _ => return Err(format!("Hata [{}:{}]: 'for .. in' yalnızca diziler, dilimler veya metinler üzerinde yinelenebilir", span.line, span.col)),
                };

                for item in items {
                    self.env.enter_scope();
                    self.env.declare(var_name.clone(), item, false);
                    let flow = self.eval_block(body);
                    self.env.exit_scope();
                    match flow? {
                        ControlFlow::Normal(_) | ControlFlow::Continue => {}
                        ControlFlow::Break => break,
                        ret @ ControlFlow::Return(_) => return Ok(ret),
                    }
                }
                Ok(ControlFlow::Normal(Value::Void))
            }
            Stmt::Break(_) => Ok(ControlFlow::Break),
            Stmt::Continue(_) => Ok(ControlFlow::Continue),
            Stmt::Return(maybe_expr, _) => {
                let val = if let Some(expr) = maybe_expr {
                    let v = self.eval_expr(expr)?;
                    if let Some(early) = self.early_return.take() {
                        return Ok(ControlFlow::Return(early));
                    }
                    v
                } else {
                    Value::Void
                };
                Ok(ControlFlow::Return(val))
            }
            Stmt::Expr(Expr::If { condition, then_branch, else_branch, .. }) => {
                let cond_val = self.eval_expr(condition)?;
                if let Some(early) = self.early_return.take() {
                    return Ok(ControlFlow::Return(early));
                }
                if Self::is_truthy(&cond_val) {
                    self.eval_block(then_branch)
                } else if let Some(eb) = else_branch {
                    self.eval_block(eb)
                } else {
                    Ok(ControlFlow::Normal(Value::Void))
                }
            }
            Stmt::Expr(Expr::Block(block)) | Stmt::Expr(Expr::UnsafeBlock(block, _)) => {
                self.eval_block(block)
            }
            Stmt::Expr(Expr::Match { subject, arms, span }) => {
                let subj_val = self.eval_expr(subject)?;
                if let Some(early) = self.early_return.take() {
                    return Ok(ControlFlow::Return(early));
                }
                for arm in arms {
                    if let Some(bindings) = Self::check_pattern(&arm.pattern, &subj_val) {
                        self.env.enter_scope();
                        for (var_name, val) in bindings {
                            self.env.declare(var_name, val, false);
                        }
                        let res = self.eval_stmt(&Stmt::Expr(arm.body.clone()));
                        self.env.exit_scope();
                        return res;
                    }
                }
                Err(format!("Hata [{}:{}]: 'match' ifadesinde eşleşen hiçbir desen bulunamadı", span.line, span.col))
            }
            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
                if let Some(early) = self.early_return.take() {
                    return Ok(ControlFlow::Return(early));
                }
                Ok(ControlFlow::Normal(Value::Void))
            }
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Int(i, _) => Ok(Value::Int(*i)),
            Expr::Float(f, _) => Ok(Value::Float(*f)),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Str(s, _) => Ok(Value::Str(s.clone())),
            Expr::FormatString(parts, _) => {
                let mut res = String::new();
                for part in parts {
                    match part {
                        FormatPart::Lit(s) => res.push_str(s),
                        FormatPart::Expr(e) => {
                            let val = self.eval_expr(e)?;
                            res.push_str(&val.to_string());
                        }
                    }
                }
                Ok(Value::Str(res))
            }
            Expr::Char(c, _) => Ok(Value::Char(*c)),
            Expr::U8(b, _) => Ok(Value::U8(*b)),
            Expr::Ident(name, span) => {
                if name == "None" {
                    return Ok(Value::None);
                }
                if let Ok(v) = self.env.get(name, *span) {
                    return Ok(v);
                }
                if let Some(c_val) = self.constants.get(name) {
                    return Ok(c_val.clone());
                }
                self.env.get(name, *span)
            }
            Expr::Unary { op, expr, span } => {
                let inner = self.eval_expr(expr)?;
                if let Value::Struct { ref name, .. } = inner {
                    let m_name = match op {
                        UnOp::Neg => "neg",
                        UnOp::Not => "not",
                    };
                    if self.methods.get(name).and_then(|m| m.get(m_name)).is_some() {
                        return self.call_struct_method(&inner, m_name, vec![], *span);
                    }
                }
                match op {
                    UnOp::Neg => match inner {
                        Value::Int(i) => Ok(Value::Int(-i)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(format!("Hata [{}:{}]: '-' operatörü yalnızca sayı tipleriyle kullanılabilir", span.line, span.col)),
                    },
                    UnOp::Not => match inner {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        _ => Err(format!("Hata [{}:{}]: '!' operatörü yalnızca bool tipleriyle kullanılabilir", span.line, span.col)),
                    },
                }
            }
            Expr::Binary { op, left, right, span } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;

                // Operator overloading for structs
                if let Value::Struct { ref name, .. } = left_val {
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
                        if self.methods.get(name).and_then(|map| map.get(m)).is_some() {
                            return self.call_struct_method(&left_val, m, vec![right_val], *span);
                        } else if *op == BinOp::NotEq {
                            if self.methods.get(name).and_then(|map| map.get("eq")).is_some() {
                                let res = self.call_struct_method(&left_val, "eq", vec![right_val], *span)?;
                                return match res {
                                    Value::Bool(b) => Ok(Value::Bool(!b)),
                                    other => Ok(other),
                                };
                            }
                        }
                    }
                }

                match op {
                    BinOp::Add => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                        (Value::U8(a), Value::U8(b)) => Ok(Value::U8(a.wrapping_add(b))),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                        (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                        (Value::Str(a), other) => Ok(Value::Str(format!("{}{}", a, other))),
                        (other, Value::Str(b)) => Ok(Value::Str(format!("{}{}", other, b))),
                        _ => Err(format!("Hata [{}:{}]: '+' için geçersiz tipler", span.line, span.col)),
                    },
                    BinOp::Sub => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                        (Value::U8(a), Value::U8(b)) => Ok(Value::U8(a.wrapping_sub(b))),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                        _ => Err(format!("Hata [{}:{}]: '-' için geçersiz tipler", span.line, span.col)),
                    },
                    BinOp::Mul => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                        (Value::U8(a), Value::U8(b)) => Ok(Value::U8(a.wrapping_mul(b))),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                        _ => Err(format!("Hata [{}:{}]: '*' için geçersiz tipler", span.line, span.col)),
                    },
                    BinOp::Div => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => {
                            if b == 0 {
                                return Err(format!("Hata [{}:{}]: Sıfıra bölme hatası!", span.line, span.col));
                            }
                            Ok(Value::Int(a / b))
                        }
                        (Value::U8(a), Value::U8(b)) => {
                            if b == 0 {
                                return Err(format!("Hata [{}:{}]: Sıfıra bölme hatası!", span.line, span.col));
                            }
                            Ok(Value::U8(a / b))
                        }
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                        _ => Err(format!("Hata [{}:{}]: '/' için geçersiz tipler", span.line, span.col)),
                    },
                    BinOp::Mod => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => {
                            if b == 0 {
                                return Err(format!("Hata [{}:{}]: Sıfıra bölme hatası!", span.line, span.col));
                            }
                            Ok(Value::Int(a % b))
                        }
                        (Value::U8(a), Value::U8(b)) => {
                            if b == 0 {
                                return Err(format!("Hata [{}:{}]: Sıfıra bölme hatası!", span.line, span.col));
                            }
                            Ok(Value::U8(a % b))
                        }
                        _ => Err(format!("Hata [{}:{}]: '%' yalnızca tamsayılar için geçerlidir", span.line, span.col)),
                    },
                    BinOp::Eq => Ok(Value::Bool(left_val == right_val)),
                    BinOp::NotEq => Ok(Value::Bool(left_val != right_val)),
                    BinOp::Lt => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                        (Value::U8(a), Value::U8(b)) => Ok(Value::Bool(a < b)),
                        (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a < b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                        _ => Err(format!("Hata [{}:{}]: '<' için uyumsuz tipler", span.line, span.col)),
                    },
                    BinOp::LtEq => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                        (Value::U8(a), Value::U8(b)) => Ok(Value::Bool(a <= b)),
                        (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a <= b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                        _ => Err(format!("Hata [{}:{}]: '<=' için uyumsuz tipler", span.line, span.col)),
                    },
                    BinOp::Gt => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                        (Value::U8(a), Value::U8(b)) => Ok(Value::Bool(a > b)),
                        (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a > b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                        _ => Err(format!("Hata [{}:{}]: '>' için uyumsuz tipler", span.line, span.col)),
                    },
                    BinOp::GtEq => match (left_val, right_val) {
                        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                        (Value::U8(a), Value::U8(b)) => Ok(Value::Bool(a >= b)),
                        (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a >= b)),
                        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                        _ => Err(format!("Hata [{}:{}]: '>=' için uyumsuz tipler", span.line, span.col)),
                    },
                    BinOp::And => match (left_val, right_val) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                        _ => Err(format!("Hata [{}:{}]: '&&' yalnızca boolean tipler için geçerlidir", span.line, span.col)),
                    },
                    BinOp::Or => match (left_val, right_val) {
                        (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                        _ => Err(format!("Hata [{}:{}]: '||' yalnızca boolean tipler için geçerlidir", span.line, span.col)),
                    },
                }
            }
            Expr::Assign { target, value, span } => {
                let evaluated = self.eval_expr(value)?;
                self.env.assign(target, evaluated.clone(), *span)?;
                Ok(evaluated)
            }
            Expr::Call { callee, args, span, .. } => {
                let mut evaluated_args = Vec::new();
                for a in args {
                    evaluated_args.push(self.eval_expr(a)?);
                }

                // Check if calling a variable containing a closure
                if let Some(val) = self.env.get_opt(callee) {
                    if let Value::Closure { params, body, env: captured_env } = val {
                        if params.len() != evaluated_args.len() {
                            return Err(format!(
                                "Hata [{}:{}]: Closure {} argüman beklerken {} argüman verildi",
                                span.line, span.col, params.len(), evaluated_args.len()
                            ));
                        }
                        let prev_env = std::mem::replace(&mut self.env, captured_env.clone());
                        self.env.enter_scope();
                        for (p, a_val) in params.iter().zip(evaluated_args.into_iter()) {
                            self.env.declare(p.name.clone(), a_val, p.is_mut);
                        }
                        let result = match self.eval_block(&body)? {
                            ControlFlow::Normal(v) | ControlFlow::Return(v) => v,
                            ControlFlow::Break | ControlFlow::Continue => {
                                return Err(format!("Hata [{}:{}]: Closure içinde 'break' veya 'continue' kullanılamaz", span.line, span.col));
                            }
                        };
                        self.env.exit_scope();
                        self.env = prev_env;
                        return Ok(result);
                    }
                }

                self.call_function(callee, evaluated_args, *span)
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                let cond_val = self.eval_expr(condition)?;
                let cf = if Self::is_truthy(&cond_val) {
                    self.eval_block(then_branch)?
                } else if let Some(eb) = else_branch {
                    self.eval_block(eb)?
                } else {
                    ControlFlow::Normal(Value::Void)
                };
                match cf {
                    ControlFlow::Normal(v) => Ok(v),
                    ControlFlow::Return(v) => Ok(v),
                    ControlFlow::Break | ControlFlow::Continue => {
                        Err("Hata: Değer beklenen ifade bağlamında 'break' veya 'continue' kullanılamaz".to_string())
                    }
                }
            }
            Expr::Array(elements, _) => {
                let mut evaluated = Vec::new();
                for e in elements {
                    evaluated.push(self.eval_expr(e)?);
                }
                Ok(Value::Array(Rc::new(RefCell::new(evaluated))))
            }
            Expr::Index { target, index, span } => {
                let target_val = self.eval_expr(target)?;
                let index_val = self.eval_expr(index)?;

                if let Value::Struct { ref name, .. } = target_val {
                    if self.methods.get(name).and_then(|m| m.get("index")).is_some() {
                        return self.call_struct_method(&target_val, "index", vec![index_val], *span);
                    }
                }

                let idx = match index_val {
                    Value::Int(i) => i,
                    _ => return Err(format!("Hata [{}:{}]: Dizi indeksi tamsayı (int) olmalıdır", span.line, span.col)),
                };

                match target_val {
                    Value::Array(arr) => {
                        let inner = arr.borrow();
                        if idx < 0 || idx as usize >= inner.len() {
                            return Err(format!(
                                "Hata [{}:{}]: Dizi sınır aşımı! Boyut: {}, Erişilmek istenen indeks: {}",
                                span.line, span.col, inner.len(), idx
                            ));
                        }
                        Ok(inner[idx as usize].clone())
                    }
                    Value::Slice { target, start, len } => {
                        let inner = target.borrow();
                        if idx < 0 || idx as usize >= len {
                            return Err(format!(
                                "Hata [{}:{}]: Dilim sınır aşımı! Boyut: {}, Erişilmek istenen indeks: {}",
                                span.line, span.col, len, idx
                            ));
                        }
                        Ok(inner[start + idx as usize].clone())
                    }
                    Value::Str(s) => {
                        let chars: Vec<char> = s.chars().collect();
                        if idx < 0 || idx as usize >= chars.len() {
                            return Err(format!(
                                "Hata [{}:{}]: Metin sınır aşımı! Boyut: {}, Erişilmek istenen indeks: {}",
                                span.line, span.col, chars.len(), idx
                            ));
                        }
                        Ok(Value::Str(chars[idx as usize].to_string()))
                    }
                    Value::StrSlice { target, start, len } => {
                        let chars: Vec<char> = target[start..start + len].chars().collect();
                        if idx < 0 || idx as usize >= chars.len() {
                            return Err(format!(
                                "Hata [{}:{}]: Metin dilimi sınır aşımı! Boyut: {}, Erişilmek istenen indeks: {}",
                                span.line, span.col, chars.len(), idx
                            ));
                        }
                        Ok(Value::Str(chars[idx as usize].to_string()))
                    }
                    _ => Err(format!("Hata [{}:{}]: Yalnızca diziler, dilimler ve metinler indekslenebilir", span.line, span.col)),
                }
            }
            Expr::Slice { target, start, end, span } => {
                let target_val = self.eval_expr(target)?;
                let start_idx = if let Some(s) = start {
                    match self.eval_expr(s)? {
                        Value::Int(i) => i,
                        _ => return Err(format!("Hata [{}:{}]: Dilim başlangıç indeksi tamsayı (int) olmalıdır", span.line, span.col)),
                    }
                } else {
                    0
                };

                match target_val {
                    Value::Array(arr) => {
                        let total_len = arr.borrow().len() as i64;
                        let end_idx = if let Some(e) = end {
                            match self.eval_expr(e)? {
                                Value::Int(i) => i,
                                _ => return Err(format!("Hata [{}:{}]: Dilim bitiş indeksi tamsayı (int) olmalıdır", span.line, span.col)),
                            }
                        } else {
                            total_len
                        };

                        if start_idx < 0 || end_idx < start_idx || end_idx > total_len {
                            return Err(format!(
                                "Hata [{}:{}]: Geçersiz dilim aralığı [{}..{}]! Dizi boyutu: {}",
                                span.line, span.col, start_idx, end_idx, total_len
                            ));
                        }

                        Ok(Value::Slice {
                            target: arr.clone(),
                            start: start_idx as usize,
                            len: (end_idx - start_idx) as usize,
                        })
                    }
                    Value::Slice { target: arr, start: base_start, len: base_len } => {
                        let total_len = base_len as i64;
                        let end_idx = if let Some(e) = end {
                            match self.eval_expr(e)? {
                                Value::Int(i) => i,
                                _ => return Err(format!("Hata [{}:{}]: Dilim bitiş indeksi tamsayı (int) olmalıdır", span.line, span.col)),
                            }
                        } else {
                            total_len
                        };

                        if start_idx < 0 || end_idx < start_idx || end_idx > total_len {
                            return Err(format!(
                                "Hata [{}:{}]: Geçersiz dilim aralığı [{}..{}]! Dilim boyutu: {}",
                                span.line, span.col, start_idx, end_idx, total_len
                            ));
                        }

                        Ok(Value::Slice {
                            target: arr.clone(),
                            start: base_start + start_idx as usize,
                            len: (end_idx - start_idx) as usize,
                        })
                    }
                    Value::Str(s) => {
                        let total_len = s.len() as i64;
                        let end_idx = if let Some(e) = end {
                            match self.eval_expr(e)? {
                                Value::Int(i) => i,
                                _ => return Err(format!("Hata [{}:{}]: Dilim bitiş indeksi tamsayı (int) olmalıdır", span.line, span.col)),
                            }
                        } else {
                            total_len
                        };

                        if start_idx < 0 || end_idx < start_idx || end_idx > total_len {
                            return Err(format!(
                                "Hata [{}:{}]: Geçersiz metin dilim aralığı [{}..{}]! Metin boyutu: {}",
                                span.line, span.col, start_idx, end_idx, total_len
                            ));
                        }

                        Ok(Value::StrSlice {
                            target: Rc::new(s),
                            start: start_idx as usize,
                            len: (end_idx - start_idx) as usize,
                        })
                    }
                    Value::StrSlice { target, start: base_start, len: base_len } => {
                        let total_len = base_len as i64;
                        let end_idx = if let Some(e) = end {
                            match self.eval_expr(e)? {
                                Value::Int(i) => i,
                                _ => return Err(format!("Hata [{}:{}]: Dilim bitiş indeksi tamsayı (int) olmalıdır", span.line, span.col)),
                            }
                        } else {
                            total_len
                        };

                        if start_idx < 0 || end_idx < start_idx || end_idx > total_len {
                            return Err(format!(
                                "Hata [{}:{}]: Geçersiz metin dilim aralığı [{}..{}]! Metin boyutu: {}",
                                span.line, span.col, start_idx, end_idx, total_len
                            ));
                        }

                        Ok(Value::StrSlice {
                            target: target.clone(),
                            start: base_start + start_idx as usize,
                            len: (end_idx - start_idx) as usize,
                        })
                    }
                    _ => Err(format!("Hata [{}:{}]: Yalnızca diziler, dilimler ve metinler dilimlenebilir", span.line, span.col)),
                }
            }
            Expr::IndexAssign { target, index, value, span } => {
                // Mutability check: if target is a variable, check if it is declared mut
                if let Expr::Ident(ref var_name, ident_span) = **target {
                    if !self.env.is_mutable(var_name) {
                        return Err(format!(
                            "Hata [{}:{}]: Değiştirilemez (immutable) '{}' dizisine eleman atanamaz! 'let mut {}' şeklinde tanımlamalısın.",
                            ident_span.line, ident_span.col, var_name, var_name
                        ));
                    }
                }

                let target_val = self.eval_expr(target)?;
                let index_val = self.eval_expr(index)?;
                let new_val = self.eval_expr(value)?;

                if let Value::Struct { ref name, .. } = target_val {
                    if self.methods.get(name).and_then(|m| m.get("index_set")).is_some() {
                        return self.call_struct_method(&target_val, "index_set", vec![index_val, new_val], *span);
                    }
                }

                let idx = match index_val {
                    Value::Int(i) => i,
                    _ => return Err(format!("Hata [{}:{}]: Dizi indeksi tamsayı (int) olmalıdır", span.line, span.col)),
                };

                match target_val {
                    Value::Array(arr) => {
                        let mut inner = arr.borrow_mut();
                        if idx < 0 || idx as usize >= inner.len() {
                            return Err(format!(
                                "Hata [{}:{}]: Dizi sınır aşımı! Boyut: {}, Atanmak istenen indeks: {}",
                                span.line, span.col, inner.len(), idx
                            ));
                        }
                        inner[idx as usize] = new_val.clone();
                        Ok(new_val)
                    }
                    _ => Err(format!("Hata [{}:{}]: Yalnızca dizilerin indekslerine atama yapılabilir", span.line, span.col)),
                }
            }
            Expr::Tuple(elements, _) => {
                let mut values = Vec::new();
                for e in elements {
                    values.push(self.eval_expr(e)?);
                }
                Ok(Value::Tuple(Rc::new(RefCell::new(values))))
            }
            Expr::TupleIndex { target, index, span } => {
                let target_val = self.eval_expr(target)?;
                match target_val {
                    Value::Tuple(tuple_elems) => {
                        let inner = tuple_elems.borrow();
                        if *index < inner.len() {
                            Ok(inner[*index].clone())
                        } else {
                            Err(format!(
                                "Hata [{}:{}]: Demet sınır aşımı! Boyut: {}, Erişilmek istenen indeks: {}",
                                span.line, span.col, inner.len(), index
                            ))
                        }
                    }
                    _ => Err(format!("Hata [{}:{}]: Yalnızca demetler demet indeksi ile indekslenebilir", span.line, span.col)),
                }
            }
            Expr::TupleIndexAssign { target, index, value, span } => {
                if let Expr::Ident(ref var_name, ident_span) = **target {
                    if !self.env.is_mutable(var_name) {
                        return Err(format!(
                            "Hata [{}:{}]: Değiştirilemez (immutable) '{}' demetine eleman atanamaz! 'let mut {}' şeklinde tanımlamalısın.",
                            ident_span.line, ident_span.col, var_name, var_name
                        ));
                    }
                }

                let target_val = self.eval_expr(target)?;
                let new_val = self.eval_expr(value)?;

                match target_val {
                    Value::Tuple(tuple_elems) => {
                        let mut inner = tuple_elems.borrow_mut();
                        if *index < inner.len() {
                            inner[*index] = new_val.clone();
                            Ok(new_val)
                        } else {
                            Err(format!(
                                "Hata [{}:{}]: Demet sınır aşımı! Boyut: {}, Atanmak istenen indeks: {}",
                                span.line, span.col, inner.len(), index
                            ))
                        }
                    }
                    _ => Err(format!("Hata [{}:{}]: Yalnızca demetlerin elemanlarına atama yapılabilir", span.line, span.col)),
                }
            }
            Expr::StructInit { name, fields, span, .. } => {
                if !self.structs.contains_key(name) {
                    return Err(format!("Hata [{}:{}]: Tanımsız struct '{}'", span.line, span.col, name));
                }

                let mut field_map = HashMap::new();
                for (f_name, f_expr) in fields {
                    let val = self.eval_expr(f_expr)?;
                    field_map.insert(f_name.clone(), val);
                }

                Ok(Value::Struct {
                    name: name.clone(),
                    fields: Rc::new(RefCell::new(field_map)),
                })
            }
            Expr::FieldAccess { target, field, span } => {
                let mut target_val = self.eval_expr(target)?;
                while let Value::Box(inner) | Value::Rc(inner, _) | Value::RefCell(inner) = target_val {
                    let next = inner.borrow().clone();
                    target_val = next;
                }
                match target_val {
                    Value::Struct { ref name, ref fields } => {
                        let inner = fields.borrow();
                        match inner.get(field) {
                            Some(val) => Ok(val.clone()),
                            None => Err(format!("Hata [{}:{}]: '{}' struct'ında '{}' adında bir alan bulunamadı", span.line, span.col, name, field)),
                        }
                    }
                    _ => Err(format!("Hata [{}:{}]: Alan erişimi (.) yalnızca struct nesnelerinde kullanılabilir", span.line, span.col)),
                }
            }
            Expr::FieldAssign { target, field, value, span } => {
                // Mutability check: if target is a variable, verify it is mutable
                if let Expr::Ident(ref var_name, ident_span) = **target {
                    if !self.env.is_mutable(var_name) {
                        return Err(format!(
                            "Hata [{}:{}]: Değiştirilemez (immutable) '{}' nesnesinin alanları değiştirilemez! 'let mut {}' şeklinde tanımlamalısın.",
                            ident_span.line, ident_span.col, var_name, var_name
                        ));
                    }
                }

                let target_val = self.eval_expr(target)?;
                let new_val = self.eval_expr(value)?;

                let mut current = target_val;
                loop {
                    match current {
                        Value::Box(inner) | Value::RefCell(inner) => {
                            let is_struct = matches!(&*inner.borrow(), Value::Struct { .. });
                            if is_struct {
                                if let Value::Struct { fields, .. } = &*inner.borrow() {
                                    fields.borrow_mut().insert(field.clone(), new_val.clone());
                                    return Ok(new_val);
                                }
                            }
                            let next = inner.borrow().clone();
                            current = next;
                        }
                        Value::Struct { ref name, ref fields } => {
                            let mut inner = fields.borrow_mut();
                            if !inner.contains_key(field) {
                                return Err(format!("Hata [{}:{}]: '{}' struct'ında '{}' alanı bulunmuyor", span.line, span.col, name, field));
                            }
                            inner.insert(field.clone(), new_val.clone());
                            return Ok(new_val);
                        }
                        _ => return Err(format!("Hata [{}:{}]: Yalnızca struct nesnelerinin alanlarına atama yapılabilir", span.line, span.col)),
                    }
                }
            }
            Expr::MethodCall { target, method, args, span, .. } => {
                let target_val = self.eval_expr(target)?;
                match &target_val {
                    Value::Some(val) => {
                        match method.as_str() {
                            "is_some" => Ok(Value::Bool(true)),
                            "is_none" => Ok(Value::Bool(false)),
                            "unwrap" => Ok((**val).clone()),
                            "unwrap_or" => Ok((**val).clone()),
                            _ => Err(format!("Hata [{}:{}]: 'Option' için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Value::None => {
                        match method.as_str() {
                            "is_some" => Ok(Value::Bool(false)),
                            "is_none" => Ok(Value::Bool(true)),
                            "unwrap" => Err(format!("Hata [{}:{}]: None değeri unwrap() edilemez", span.line, span.col)),
                            "unwrap_or" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'unwrap_or()' 1 varsayılan değer bekler", span.line, span.col)); }
                                self.eval_expr(&args[0])
                            }
                            _ => Err(format!("Hata [{}:{}]: 'Option' için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Value::Ok(val) => {
                        match method.as_str() {
                            "is_ok" => Ok(Value::Bool(true)),
                            "is_err" => Ok(Value::Bool(false)),
                            "unwrap" => Ok((**val).clone()),
                            "unwrap_or" => Ok((**val).clone()),
                            _ => Err(format!("Hata [{}:{}]: 'Result' için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Value::Err(err) => {
                        match method.as_str() {
                            "is_ok" => Ok(Value::Bool(false)),
                            "is_err" => Ok(Value::Bool(true)),
                            "unwrap" => Err(format!("Hata [{}:{}]: Err değeri unwrap() edilemez: {}", span.line, span.col, err)),
                            "unwrap_or" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'unwrap_or()' 1 varsayılan değer bekler", span.line, span.col)); }
                                self.eval_expr(&args[0])
                            }
                            _ => Err(format!("Hata [{}:{}]: 'Result' için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Value::Str(s) => {
                        match method.as_str() {
                            "len" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'len()' argüman almaz", span.line, span.col)); }
                                return Ok(Value::Int(s.len() as i64));
                            }
                            "trim" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'trim()' argüman almaz", span.line, span.col)); }
                                return Ok(Value::Str(s.trim().to_string()));
                            }
                            "to_upper" | "to_uppercase" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'to_upper()' argüman almaz", span.line, span.col)); }
                                return Ok(Value::Str(s.to_uppercase()));
                            }
                            "to_lower" | "to_lowercase" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'to_lower()' argüman almaz", span.line, span.col)); }
                                return Ok(Value::Str(s.to_lowercase()));
                            }
                            "contains" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'contains()' 1 argüman bekler", span.line, span.col)); }
                                let sub = self.eval_expr(&args[0])?;
                                let sub_str = match sub { Value::Str(v) => v, other => other.to_string() };
                                return Ok(Value::Bool(s.contains(&sub_str)));
                            }
                            "split" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'split()' 1 argüman bekler", span.line, span.col)); }
                                let sep = self.eval_expr(&args[0])?;
                                let sep_str = match sep { Value::Str(v) => v, other => other.to_string() };
                                let parts: Vec<Value> = s.split(&sep_str).map(|p| Value::Str(p.to_string())).collect();
                                return Ok(Value::Array(Rc::new(RefCell::new(parts))));
                            }
                            "replace" => {
                                if args.len() != 2 { return Err(format!("Hata [{}:{}]: 'replace()' 2 argüman bekler", span.line, span.col)); }
                                let from_v = self.eval_expr(&args[0])?;
                                let to_v = self.eval_expr(&args[1])?;
                                let from_s = match from_v { Value::Str(v) => v, other => other.to_string() };
                                let to_s = match to_v { Value::Str(v) => v, other => other.to_string() };
                                return Ok(Value::Str(s.replace(&from_s, &to_s)));
                            }
                            "char_at" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'char_at()' 1 argüman bekler", span.line, span.col)); }
                                let idx_v = self.eval_expr(&args[0])?;
                                let idx = match idx_v { Value::Int(i) => i, _ => return Err(format!("Hata [{}:{}]: 'char_at()' int indeks bekler", span.line, span.col)) };
                                if idx < 0 { return Err(format!("Hata [{}:{}]: Negatif indeks geçersiz", span.line, span.col)); }
                                return match s.chars().nth(idx as usize) {
                                    Some(c) => Ok(Value::Char(c)),
                                    None => Err(format!("Hata [{}:{}]: Metin indeksi sınır dışı", span.line, span.col)),
                                };
                            }
                            _ => return Err(format!("Hata [{}:{}]: 'str' için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Value::Array(arr) => {
                        match method.as_str() {
                            "len" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'len()' argüman almaz", span.line, span.col)); }
                                return Ok(Value::Int(arr.borrow().len() as i64));
                            }
                            "push" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'push()' 1 argüman bekler", span.line, span.col)); }
                                let val = self.eval_expr(&args[0])?;
                                arr.borrow_mut().push(val);
                                return Ok(Value::Void);
                            }
                            "pop" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'pop()' argüman almaz", span.line, span.col)); }
                                match arr.borrow_mut().pop() {
                                    Some(v) => return Ok(v),
                                    None => return Err(format!("Hata [{}:{}]: Boş diziden 'pop' yapılamaz", span.line, span.col)),
                                }
                            }
                            "map" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'map()' 1 closure argümanı bekler", span.line, span.col)); }
                                let closure_val = self.eval_expr(&args[0])?;
                                let (params, body, captured_env) = match closure_val {
                                    Value::Closure { params, body, env } => (params, body, env),
                                    _ => return Err(format!("Hata [{}:{}]: 'map()' closure bekler", span.line, span.col)),
                                };
                                let elements = arr.borrow().clone();
                                let mut result = Vec::new();
                                for elem in elements {
                                    let prev_env = std::mem::replace(&mut self.env, captured_env.clone());
                                    self.env.enter_scope();
                                    if let Some(first_param) = params.first() {
                                        self.env.declare(first_param.name.clone(), elem, first_param.is_mut);
                                    }
                                    let mapped = match self.eval_block(&body)? {
                                        ControlFlow::Normal(v) | ControlFlow::Return(v) => v,
                                        _ => return Err(format!("Hata [{}:{}]: 'map' içinde break/continue kullanılamaz", span.line, span.col)),
                                    };
                                    self.env.exit_scope();
                                    self.env = prev_env;
                                    result.push(mapped);
                                }
                                return Ok(Value::Array(Rc::new(RefCell::new(result))));
                            }
                            "filter" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'filter()' 1 closure argümanı bekler", span.line, span.col)); }
                                let closure_val = self.eval_expr(&args[0])?;
                                let (params, body, captured_env) = match closure_val {
                                    Value::Closure { params, body, env } => (params, body, env),
                                    _ => return Err(format!("Hata [{}:{}]: 'filter()' closure bekler", span.line, span.col)),
                                };
                                let elements = arr.borrow().clone();
                                let mut result = Vec::new();
                                for elem in elements {
                                    let prev_env = std::mem::replace(&mut self.env, captured_env.clone());
                                    self.env.enter_scope();
                                    if let Some(first_param) = params.first() {
                                        self.env.declare(first_param.name.clone(), elem.clone(), first_param.is_mut);
                                    }
                                    let keep = match self.eval_block(&body)? {
                                        ControlFlow::Normal(v) | ControlFlow::Return(v) => Self::is_truthy(&v),
                                        _ => return Err(format!("Hata [{}:{}]: 'filter' içinde break/continue kullanılamaz", span.line, span.col)),
                                    };
                                    self.env.exit_scope();
                                    self.env = prev_env;
                                    if keep {
                                        result.push(elem);
                                    }
                                }
                                return Ok(Value::Array(Rc::new(RefCell::new(result))));
                            }
                            _ => return Err(format!("Hata [{}:{}]: Dizi için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Value::Slice { target, start, len } => {
                        let (start, len) = (*start, *len);
                        match method.as_str() {
                            "len" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'len()' argüman almaz", span.line, span.col)); }
                                return Ok(Value::Int(len as i64));
                            }
                            "to_array" | "to_vec" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'to_array()' argüman almaz", span.line, span.col)); }
                                let inner = target.borrow();
                                let cloned = inner[start..start + len].to_vec();
                                return Ok(Value::Array(Rc::new(RefCell::new(cloned))));
                            }
                            _ => return Err(format!("Hata [{}:{}]: Dilim için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Value::StrSlice { target, start, len } => {
                        let (start, len) = (*start, *len);
                        let slice_str = &target[start..start + len];
                        match method.as_str() {
                            "len" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'len()' argüman almaz", span.line, span.col)); }
                                return Ok(Value::Int(len as i64));
                            }
                            "to_str" | "to_string" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'to_str()' argüman almaz", span.line, span.col)); }
                                return Ok(Value::Str(slice_str.to_string()));
                            }
                            "trim" => return Ok(Value::Str(slice_str.trim().to_string())),
                            "to_upper" | "to_uppercase" => return Ok(Value::Str(slice_str.to_uppercase())),
                            "to_lower" | "to_lowercase" => return Ok(Value::Str(slice_str.to_lowercase())),
                            "contains" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'contains()' 1 argüman bekler", span.line, span.col)); }
                                let sub = self.eval_expr(&args[0])?;
                                let sub_str = match sub {
                                    Value::Str(s) => s,
                                    _ => return Err(format!("Hata [{}:{}]: 'contains()' metin bekler", span.line, span.col)),
                                };
                                return Ok(Value::Bool(slice_str.contains(&sub_str)));
                            }
                            "char_at" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'char_at()' 1 argüman bekler", span.line, span.col)); }
                                let idx_val = self.eval_expr(&args[0])?;
                                let idx = match idx_val {
                                    Value::Int(i) => i,
                                    _ => return Err(format!("Hata [{}:{}]: 'char_at()' tamsayı indeks bekler", span.line, span.col)),
                                };
                                let chars: Vec<char> = slice_str.chars().collect();
                                if idx < 0 || idx as usize >= chars.len() {
                                    return Err(format!("Hata [{}:{}]: İndeks sınır aşımı: {}", span.line, span.col, idx));
                                }
                                return Ok(Value::Char(chars[idx as usize]));
                            }
                            _ => return Err(format!("Hata [{}:{}]: '&str' için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Value::Map(m) => {
                        match method.as_str() {
                            "set" => {
                                if args.len() != 2 { return Err(format!("Hata [{}:{}]: 'Map.set()' 2 argüman bekler", span.line, span.col)); }
                                let k = self.eval_expr(&args[0])?;
                                let v = self.eval_expr(&args[1])?;
                                let k_str = match k { Value::Str(s) => s, other => other.to_string() };
                                m.borrow_mut().insert(k_str, v);
                                return Ok(Value::Void);
                            }
                            "get" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'Map.get()' 1 argüman bekler", span.line, span.col)); }
                                let k = self.eval_expr(&args[0])?;
                                let k_str = match k { Value::Str(s) => s, other => other.to_string() };
                                return Ok(m.borrow().get(&k_str).cloned().unwrap_or(Value::Str(String::new())));
                            }
                            "has" => {
                                if args.len() != 1 { return Err(format!("Hata [{}:{}]: 'Map.has()' 1 argüman bekler", span.line, span.col)); }
                                let k = self.eval_expr(&args[0])?;
                                let k_str = match k { Value::Str(s) => s, other => other.to_string() };
                                return Ok(Value::Bool(m.borrow().contains_key(&k_str)));
                            }
                            "len" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'Map.len()' argüman almaz", span.line, span.col)); }
                                return Ok(Value::Int(m.borrow().len() as i64));
                            }
                            "keys" => {
                                if !args.is_empty() { return Err(format!("Hata [{}:{}]: 'Map.keys()' argüman almaz", span.line, span.col)); }
                                let keys: Vec<Value> = m.borrow().keys().cloned().map(Value::Str).collect();
                                return Ok(Value::Array(Rc::new(RefCell::new(keys))));
                            }
                            _ => return Err(format!("Hata [{}:{}]: 'Map' için '{}' metodu bulunamadı", span.line, span.col, method)),
                        }
                    }
                    Value::Rc(inner, count) if method == "clone" => {
                        *count.borrow_mut() += 1;
                        Ok(Value::Rc(Rc::clone(inner), Rc::clone(count)))
                    }
                    Value::RefCell(inner) if method == "borrow" => {
                        Ok(Value::Ref { val: Rc::clone(inner), is_mut: false })
                    }
                    Value::RefCell(inner) if method == "borrow_mut" => {
                        Ok(Value::Ref { val: Rc::clone(inner), is_mut: true })
                    }
                    Value::Box(inner) | Value::Rc(inner, _) => {
                        let mut evaluated_args = Vec::new();
                        for a in args {
                            evaluated_args.push(self.eval_expr(a)?);
                        }
                        let deref_struct = inner.borrow().clone();
                        self.call_struct_method(&deref_struct, method, evaluated_args, *span)
                    }
                    Value::Struct { .. } => {
                        let mut evaluated_args = Vec::new();
                        for a in args {
                            evaluated_args.push(self.eval_expr(a)?);
                        }
                        self.call_struct_method(&target_val, method, evaluated_args, *span)
                    }
                    _ => return Err(format!("Hata [{}:{}]: Metot çağrısı (.) yalnızca struct, Map, dizi veya metinlerde kullanılabilir", span.line, span.col)),
                }
            }
            Expr::Path { base, member, span } => {
                if let Some(enum_def) = self.enums.get(base) {
                    if enum_def.variants.contains(member) {
                        return Ok(Value::EnumVariant {
                            enum_name: base.clone(),
                            variant: member.clone(),
                        });
                    } else {
                        return Err(format!(
                            "Hata [{}:{}]: '{}' enum'ında '{}' adında bir variant bulunmuyor",
                            span.line, span.col, base, member
                        ));
                    }
                }
                Err(format!(
                    "Hata [{}:{}]: Tanımsız enum veya modül '{}'",
                    span.line, span.col, base
                ))
            }
            Expr::Match { subject, arms, span } => {
                let subject_val = self.eval_expr(subject)?;
                if let Some(early) = self.early_return.take() {
                    return Ok(early);
                }

                for arm in arms {
                    if let Some(bindings) = Self::check_pattern(&arm.pattern, &subject_val) {
                        self.env.enter_scope();
                        for (var_name, val) in bindings {
                            self.env.declare(var_name, val, false);
                        }
                        let res = self.eval_expr(&arm.body);
                        self.env.exit_scope();
                        return res;
                    }
                }

                Err(format!(
                    "Hata [{}:{}]: 'match' ifadesinde eşleşen hiçbir desen (arm) bulunamadı! Değer: {}",
                    span.line, span.col, subject_val
                ))
            }
            Expr::Block(block) => {
                match self.eval_block(block)? {
                    ControlFlow::Normal(v) => Ok(v),
                    ControlFlow::Return(v) => Ok(v),
                    ControlFlow::Break | ControlFlow::Continue => {
                        Err("Hata: Değer beklenen ifade bağlamında 'break' veya 'continue' kullanılamaz".to_string())
                    }
                }
            }
            Expr::UnsafeBlock(block, _) => {
                match self.eval_block(block)? {
                    ControlFlow::Normal(v) => Ok(v),
                    ControlFlow::Return(v) => Ok(v),
                    ControlFlow::Break | ControlFlow::Continue => {
                        Err("Hata: Değer beklenen ifade bağlamında 'break' veya 'continue' kullanılamaz".to_string())
                    }
                }
            }
            Expr::Borrow { expr, is_mut, .. } => {
                let inner_val = self.eval_expr(expr)?;
                Ok(Value::Ref {
                    val: Rc::new(RefCell::new(inner_val)),
                    is_mut: *is_mut,
                })
            }
            Expr::Deref { expr, span } => {
                let v = self.eval_expr(expr)?;
                match v {
                    Value::Box(inner) | Value::Rc(inner, _) | Value::RefCell(inner) => {
                        let cloned = inner.borrow().clone();
                        Ok(cloned)
                    }
                    Value::Ref { val, .. } => {
                        let cloned = val.borrow().clone();
                        Ok(cloned)
                    }
                    _ => Err(format!("Hata [{}:{}]: '*' operatörü yalnızca referanslar veya akıllı işaretçiler (Box, Rc) üzerinde kullanılabilir", span.line, span.col)),
                }
            }
            Expr::DerefAssign { target, value, span } => {
                let val = self.eval_expr(value)?;
                let tgt = self.eval_expr(target)?;
                match tgt {
                    Value::Box(inner) | Value::RefCell(inner) => {
                        *inner.borrow_mut() = val.clone();
                        Ok(val)
                    }
                    Value::Ref { val: r_val, is_mut } => {
                        if !is_mut {
                            return Err(format!("Hata [{}:{}]: Değiştirilemez (&) referans üzerinden atama yapılamaz", span.line, span.col));
                        }
                        *r_val.borrow_mut() = val.clone();
                        Ok(val)
                    }
                    _ => Err(format!("Hata [{}:{}]: '*' operatörü ile atama yalnızca değiştirilebilir referanslar veya Box üzerinde yapılabilir", span.line, span.col)),
                }
            }
            Expr::Try(expr, span) => {
                let v = self.eval_expr(expr)?;
                if self.early_return.is_some() {
                    return Ok(Value::Void);
                }
                match v {
                    Value::Some(val) => Ok(*val),
                    Value::None => {
                        self.early_return = Some(Value::None);
                        Ok(Value::Void)
                    }
                    Value::Ok(val) => Ok(*val),
                    Value::Err(err) => {
                        self.early_return = Some(Value::Err(err));
                        Ok(Value::Void)
                    }
                    _ => Err(format!(
                        "Hata [{}:{}]: '?' operatörü yalnızca Result veya Option tipleri üzerinde kullanılabilir",
                        span.line, span.col
                    )),
                }
            }
            Expr::Closure { params, body, .. } => {
                Ok(Value::Closure {
                    params: params.clone(),
                    body: body.clone(),
                    env: self.env.clone(),
                })
            }
        }
    }

    fn check_pattern(pattern: &Pattern, subject_val: &Value) -> Option<Vec<(String, Value)>> {
        match pattern {
            Pattern::Wildcard(_) => Some(Vec::new()),
            Pattern::Int(i, _) => {
                if *subject_val == Value::Int(*i) { Some(Vec::new()) } else { None }
            }
            Pattern::Float(fl, _) => {
                if *subject_val == Value::Float(*fl) { Some(Vec::new()) } else { None }
            }
            Pattern::Bool(b, _) => {
                if *subject_val == Value::Bool(*b) { Some(Vec::new()) } else { None }
            }
            Pattern::Str(s, _) => {
                if *subject_val == Value::Str(s.clone()) { Some(Vec::new()) } else { None }
            }
            Pattern::Char(c, _) => {
                if *subject_val == Value::Char(*c) { Some(Vec::new()) } else { None }
            }
            Pattern::U8(b, _) => {
                if *subject_val == Value::U8(*b) { Some(Vec::new()) } else { None }
            }
            Pattern::EnumVariant { enum_name, variant, .. } => {
                if *subject_val == (Value::EnumVariant { enum_name: enum_name.clone(), variant: variant.clone() }) {
                    Some(Vec::new())
                } else {
                    None
                }
            }
            Pattern::Ident(name, _) => Some(vec![(name.clone(), subject_val.clone())]),
            Pattern::Some(inner, _) => {
                if let Value::Some(v) = subject_val {
                    Self::check_pattern(inner, v)
                } else {
                    None
                }
            }
            Pattern::None(_) => {
                if matches!(subject_val, Value::None) {
                    Some(Vec::new())
                } else {
                    None
                }
            }
            Pattern::Ok(inner, _) => {
                if let Value::Ok(v) = subject_val {
                    Self::check_pattern(inner, v)
                } else {
                    None
                }
            }
            Pattern::Err(inner, _) => {
                if let Value::Err(e) = subject_val {
                    Self::check_pattern(inner, e)
                } else {
                    None
                }
            }
            Pattern::Tuple(pats, _) => {
                if let Value::Tuple(elems) = subject_val {
                    let inner = elems.borrow();
                    if pats.len() == inner.len() {
                        let mut bindings = Vec::new();
                        for (p, v) in pats.iter().zip(inner.iter()) {
                            if let Some(b) = Self::check_pattern(p, v) {
                                bindings.extend(b);
                            } else {
                                return None;
                            }
                        }
                        Some(bindings)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn is_truthy(val: &Value) -> bool {
        match val {
            Value::Bool(b) => *b,
            _ => false,
        }
    }
}
