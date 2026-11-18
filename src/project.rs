use std::fs;
use std::path::{Path, PathBuf};
use crate::ast::Item;
use crate::eval::Interpreter;
use crate::loader::ModuleLoader;
use crate::span::Span;
use crate::typecheck::TypeChecker;

#[derive(Debug, Clone)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub description: Option<String>,
    pub libs: Vec<String>,
    pub lib_paths: Vec<String>,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            name: "oxide_app".to_string(),
            version: "0.1.0".to_string(),
            entry: "src/main.ox".to_string(),
            description: None,
            libs: Vec::new(),
            lib_paths: Vec::new(),
        }
    }
}

fn parse_string_array(s: &str) -> Vec<String> {
    let s = s.trim();
    if s.starts_with('[') && s.ends_with(']') {
        s[1..s.len() - 1]
            .split(',')
            .map(|item| item.trim().trim_matches('"').trim_matches('\'').trim().to_string())
            .filter(|item| !item.is_empty())
            .collect()
    } else if !s.is_empty() {
        vec![s.trim_matches('"').trim_matches('\'').trim().to_string()]
    } else {
        Vec::new()
    }
}

/// Parses an `oxide.toml` file without any external crate dependencies.
pub fn parse_oxide_toml(content: &str) -> PackageConfig {
    let mut config = PackageConfig::default();
    let mut current_section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].trim().to_lowercase();
            continue;
        }

        if let Some((key, val)) = trimmed.split_once('=') {
            let k = key.trim().to_lowercase();
            let mut v = val.trim();
            if v.starts_with('"') && v.ends_with('"') && v.len() >= 2 {
                v = &v[1..v.len() - 1];
            }

            if current_section == "package" || current_section.is_empty() {
                match k.as_str() {
                    "name" => config.name = v.to_string(),
                    "version" => config.version = v.to_string(),
                    "entry" => config.entry = v.to_string(),
                    "description" => config.description = Some(v.to_string()),
                    _ => {}
                }
            } else if current_section == "dependencies" || current_section == "libs" {
                match k.as_str() {
                    "libs" => config.libs.extend(parse_string_array(val)),
                    "lib_paths" => config.lib_paths.extend(parse_string_array(val)),
                    _ => {}
                }
            }
        }
    }

    config
}

/// Discovers `oxide.toml` in current or parent directories.
pub fn find_oxide_toml() -> Option<(PathBuf, PackageConfig)> {
    let mut cur = std::env::current_dir().ok()?;
    loop {
        let toml_path = cur.join("oxide.toml");
        if toml_path.exists() {
            if let Ok(content) = fs::read_to_string(&toml_path) {
                let cfg = parse_oxide_toml(&content);
                return Some((toml_path, cfg));
            }
        }
        if !cur.pop() {
            break;
        }
    }
    None
}

/// Scaffolds a new Oxide project (`oxide new <name>`).
pub fn init_new_project(name: &str) -> Result<(), String> {
    let root = Path::new(name);
    if root.exists() {
        return Err(format!("'{}' dizini zaten mevcut", name));
    }

    let pkg_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(name);

    fs::create_dir_all(root.join("src")).map_err(|e| e.to_string())?;
    fs::create_dir_all(root.join("tests")).map_err(|e| e.to_string())?;

    let toml_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
entry = "src/main.ox"
description = "Oxide ile yazılmış yeni bir proje"
"#,
        pkg_name
    );
    fs::write(root.join("oxide.toml"), toml_content).map_err(|e| e.to_string())?;

    let gitignore_content = "/target/\n";
    fs::write(root.join(".gitignore"), gitignore_content).map_err(|e| e.to_string())?;

    let main_ox = r#"fn main() -> void {
    println("Merhaba, Oxide!");
}
"#;
    fs::write(root.join("src/main.ox"), main_ox).map_err(|e| e.to_string())?;

    let test_ox = r#"fn test_ornek() -> void {
    let x = 10 + 20;
    assert(x == 30, "10 + 20 esit olmali 30");
}
"#;
    fs::write(root.join("tests/test_basic.ox"), test_ox).map_err(|e| e.to_string())?;

    let cfg = PackageConfig {
        name: pkg_name.to_string(),
        version: "0.1.0".to_string(),
        entry: "src/main.ox".to_string(),
        description: Some("Oxide ile yazılmış yeni bir proje".to_string()),
        libs: Vec::new(),
        lib_paths: Vec::new(),
    };
    update_lockfile(root, &cfg)?;

    println!("✔ '{}' Oxide projesi başarıyla oluşturuldu!", name);
    println!("  ├── oxide.toml");
    println!("  ├── oxide.lock");
    println!("  ├── .gitignore");
    println!("  ├── src/main.ox");
    println!("  └── tests/test_basic.ox");
    println!("\nBaşlamak için:\n  cd {}\n  oxide run\n  oxide build", name);

    Ok(())
}

/// Creates or updates `oxide.lock` for the given project.
pub fn update_lockfile(project_dir: &Path, config: &PackageConfig) -> Result<(), String> {
    let lock_path = project_dir.join("oxide.lock");
    let content = format!(
        r#"# Bu dosya Oxide tarafından otomatik üretilmiştir.
# El ile düzenlenmesi önerilmez.
version = 1

[[package]]
name = "{}"
version = "{}"
entry = "{}"
source = "local"
"#,
        config.name, config.version, config.entry
    );
    fs::write(&lock_path, content).map_err(|e| format!("oxide.lock yazılamadı: {}", e))?;
    Ok(())
}

/// Cleans the `target/` build directory.
pub fn clean_project() -> Result<(), String> {
    let target_dir = if let Some((toml_path, _)) = find_oxide_toml() {
        toml_path.parent().unwrap_or(Path::new(".")).join("target")
    } else {
        PathBuf::from("target")
    };

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).map_err(|e| format!("'target/' dizini silinemedi: {}", e))?;
        println!("✔ '{}' dizini başarıyla temizlendi.", target_dir.display());
    } else {
        println!("✔ Temizlenecek 'target/' dizini bulunamadı.");
    }
    Ok(())
}

/// Ensures `target/.gitignore` exists so build artifacts are ignored by git.
pub fn ensure_target_gitignore(target_root: &Path) {
    let gitignore_path = target_root.join(".gitignore");
    if !gitignore_path.exists() {
        let _ = fs::write(gitignore_path, "*\n");
    }
}

/// Runs test functions in a file or project.
pub fn run_tests(target: Option<&str>) -> bool {
    let test_files = match target {
        Some(path_str) => {
            let p = Path::new(path_str);
            if p.is_file() {
                vec![p.to_path_buf()]
            } else if p.is_dir() {
                collect_ox_files(p)
            } else {
                eprintln!("Hata: '{}' yolu bulunamadı", path_str);
                return false;
            }
        }
        None => {
            // Scan current directory: tests/ first, then src/
            let mut files = Vec::new();
            let tests_dir = Path::new("tests");
            if tests_dir.is_dir() {
                files.extend(collect_ox_files(tests_dir));
            }
            let src_dir = Path::new("src");
            if src_dir.is_dir() {
                files.extend(collect_ox_files(src_dir));
            }
            if files.is_empty() {
                // Try current dir
                files.extend(collect_ox_files(Path::new(".")));
            }
            files
        }
    };

    if test_files.is_empty() {
        println!("Test çalıştırılacak hiçbir .ox dosyası bulunamadı.");
        return true;
    }

    let mut total_passed = 0;
    let mut total_failed = 0;

    for file in &test_files {
        let file_str = file.to_string_lossy();
        let mut loader = ModuleLoader::new();
        let program = match loader.load_entry(Path::new(&*file_str)) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Hata [{}]: {}", file_str, e);
                total_failed += 1;
                continue;
            }
        };

        let program = match crate::monomorph::Monomorphizer::monomorphize_program(program) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Monomorphization Hatası [{}]: {}", file_str, e);
                total_failed += 1;
                continue;
            }
        };

        let mut checker = TypeChecker::new();
        if let Err(e) = checker.check_program(&program) {
            eprintln!("Tip Hatası [{}]: {}", file_str, e);
            total_failed += 1;
            continue;
        }

        // Find all functions starting with "test"
        let mut test_names = Vec::new();
        for item in &program.items {
            if let Item::Function(f) = item {
                if f.name.starts_with("test") {
                    test_names.push(f.name.clone());
                }
            }
        }

        if test_names.is_empty() {
            continue;
        }

        println!("\nÇalıştırılıyor: {} ({} test)", file_str, test_names.len());

        for test_fn in test_names {
            print!("  test {} ... ", test_fn);
            use std::io::Write;
            let _ = std::io::stdout().flush();

            let mut interpreter = Interpreter::new();
            interpreter.load_program(program.clone());

            match interpreter.call_function(&test_fn, vec![], Span::new(1, 1)) {
                Ok(_) => {
                    println!("\x1b[32mok\x1b[0m");
                    total_passed += 1;
                }
                Err(err) => {
                    println!("\x1b[31mBAŞARISIZ (FAILED)\x1b[0m");
                    eprintln!("    Hata: {}", err);
                    total_failed += 1;
                }
            }
        }
    }

    println!("\nTest Sonucu: {} geçti; {} başarısız oldu\n", total_passed, total_failed);
    total_failed == 0
}

fn collect_ox_files(dir: &Path) -> Vec<PathBuf> {
    let mut res = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                res.extend(collect_ox_files(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("ox") {
                res.push(path);
            }
        }
    }
    res
}
