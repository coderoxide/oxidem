use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use crate::ast::{Item, Program};
use crate::lexer::Lexer;
use crate::parser::Parser;

#[derive(Debug, Clone, Default)]
pub struct ModuleCache {
    pub asts: HashMap<PathBuf, Program>,
}

#[allow(dead_code)]
impl ModuleCache {
    pub fn new() -> Self {
        Self {
            asts: HashMap::new(),
        }
    }

    pub fn get(&self, path: &Path) -> Option<&Program> {
        self.asts.get(path)
    }

    pub fn insert(&mut self, path: PathBuf, program: Program) {
        self.asts.insert(path, program);
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.asts.contains_key(path)
    }

    pub fn clear(&mut self) {
        self.asts.clear();
    }
}

pub struct ModuleLoader {
    visited: HashSet<PathBuf>,
    import_stack: Vec<PathBuf>,
    pub cache: ModuleCache,
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
            import_stack: Vec::new(),
            cache: ModuleCache::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_cache(cache: ModuleCache) -> Self {
        Self {
            visited: HashSet::new(),
            import_stack: Vec::new(),
            cache,
        }
    }

    pub fn load_entry(&mut self, entry_path: &Path) -> Result<Program, String> {
        let canon = self.resolve_file_path(entry_path, None)?;
        self.load_file(&canon)
    }

    fn resolve_file_path(&self, requested: &Path, base_dir: Option<&Path>) -> Result<PathBuf, String> {
        let full_path = if requested.is_absolute() {
            requested.to_path_buf()
        } else if let Some(base) = base_dir {
            base.join(requested)
        } else {
            requested.to_path_buf()
        };

        // Try exact path
        if full_path.is_file() {
            return fs::canonicalize(&full_path).map_err(|e| format!("Dosya yolu çözümlenemedi '{}': {}", full_path.display(), e));
        }

        // Try appending .ox if missing
        if full_path.extension().is_none() {
            let with_ox = full_path.with_extension("ox");
            if with_ox.is_file() {
                return fs::canonicalize(&with_ox).map_err(|e| format!("Dosya yolu çözümlenemedi '{}': {}", with_ox.display(), e));
            }
        }

        Err(format!("Dosya bulunamadı: '{}'", full_path.display()))
    }

    fn load_file(&mut self, file_path: &Path) -> Result<Program, String> {
        // Check for circular import in active stack
        if self.import_stack.contains(&file_path.to_path_buf()) {
            let chain = self
                .import_stack
                .iter()
                .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            let current = file_path.file_name().unwrap_or_default().to_string_lossy();
            return Err(format!(
                "Modül Hatası: Döngüsel içe aktarma (circular import) tespit edildi: {} -> {}",
                chain, current
            ));
        }

        // Check if already visited and loaded by another branch (diamond import protection)
        if self.visited.contains(file_path) {
            return Ok(Program { items: Vec::new() });
        }

        let parsed_prog = if let Some(cached) = self.cache.get(file_path) {
            cached.clone()
        } else {
            let source = fs::read_to_string(file_path)
                .map_err(|e| format!("Dosya okunamadı '{}': {}", file_path.display(), e))?;

            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize()
                .map_err(|e| format!("[{}] {}", file_path.display(), e))?;

            let mut parser = Parser::new(tokens);
            let prog = parser.parse_program()
                .map_err(|e| format!("[{}] {}", file_path.display(), e))?;

            self.cache.insert(file_path.to_path_buf(), prog.clone());
            prog
        };

        self.import_stack.push(file_path.to_path_buf());

        let base_dir = file_path.parent();
        let mut final_items = Vec::new();

        for item in parsed_prog.items {
            match item {
                Item::Import(import_str, span) => {
                    let target_path = self.resolve_file_path(Path::new(&import_str), base_dir)
                        .map_err(|_| format!("Modül Hatası [{}:{}]: İçe aktarılmak istenen '{}' dosyası bulunamadı", span.line, span.col, import_str))?;

                    let imported_prog = self.load_file(&target_path)?;
                    // Dependencies' items come first
                    final_items.extend(imported_prog.items);
                }
                other => {
                    final_items.push(other);
                }
            }
        }

        self.import_stack.pop();
        self.visited.insert(file_path.to_path_buf());

        Ok(Program { items: final_items })
    }
}
