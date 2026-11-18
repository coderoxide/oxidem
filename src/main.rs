mod span;
mod token;
mod lexer;
mod ast;
mod parser;
mod eval;
mod codegen_c;
mod typecheck;
mod loader;
mod hir;
mod borrowck;
mod opt;
mod project;
mod monomorph;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use lexer::Lexer;
use parser::Parser;
use eval::Interpreter;
use codegen_c::CodegenC;
use typecheck::TypeChecker;
use loader::ModuleLoader;
use hir::HirBuilder;
use borrowck::BorrowChecker;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "new" => {
            if args.len() < 3 {
                eprintln!("Kullanım: oxide new <proje_adi>");
                return;
            }
            if let Err(e) = project::init_new_project(&args[2]) {
                eprintln!("Hata: {}", e);
            }
        }
        "clean" => {
            if let Err(e) = project::clean_project() {
                eprintln!("Hata: {}", e);
            }
        }
        "test" => {
            let target = args.get(2).map(|s| s.as_str());
            let success = project::run_tests(target);
            if !success {
                std::process::exit(1);
            }
        }
        "run" => {
            if args.len() < 3 {
                if let Some((toml_path, cfg)) = project::find_oxide_toml() {
                    let proj_root = toml_path.parent().unwrap_or(Path::new("."));
                    let _ = project::update_lockfile(proj_root, &cfg);
                    let entry_path = proj_root.join(&cfg.entry);
                    run_file(entry_path.to_str().unwrap_or(&cfg.entry));
                    return;
                }
                eprintln!("Kullanım: oxide run <dosya.ox> (veya proje dizininde 'oxide run')");
                return;
            }
            run_file(&args[2]);
        }
        "check" => {
            if args.len() < 3 {
                if let Some((toml_path, cfg)) = project::find_oxide_toml() {
                    let proj_root = toml_path.parent().unwrap_or(Path::new("."));
                    let _ = project::update_lockfile(proj_root, &cfg);
                    let entry_path = proj_root.join(&cfg.entry);
                    check_file(entry_path.to_str().unwrap_or(&cfg.entry));
                    return;
                }
                eprintln!("Kullanım: oxide check <dosya.ox>");
                return;
            }
            check_file(&args[2]);
        }
        "build" => {
            let opt_proj = project::find_oxide_toml();
            let mut build_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };

            let has_file_arg = build_args.iter().any(|a| !a.starts_with('-'));
            if !has_file_arg {
                if let Some((_, ref cfg)) = opt_proj {
                    build_args.insert(0, cfg.entry.clone());
                } else {
                    eprintln!("Kullanım: oxide build [dosya.ox] [-o <cikti>] [--emit-c] [--release] [--cc <derleyici>]");
                    return;
                }
            }

            match parse_build_args(&build_args, opt_proj) {
                Ok(config) => build_file(config),
                Err(err) => eprintln!("{}", err),
            }
        }
        "tokens" => {
            if args.len() < 3 {
                eprintln!("Kullanım: oxide tokens <dosya.ox>");
                return;
            }
            dump_tokens(&args[2]);
        }
        "ast" => {
            if args.len() < 3 {
                eprintln!("Kullanım: oxide ast <dosya.ox>");
                return;
            }
            dump_ast(&args[2]);
        }
        "hir" => {
            if args.len() < 3 {
                eprintln!("Kullanım: oxide hir <dosya.ox> [--opt]");
                return;
            }
            let optimize = args.iter().any(|a| a == "--opt");
            let file_arg = args.iter().skip(2).find(|a| !a.starts_with("--")).cloned();
            match file_arg {
                Some(f) => dump_hir(&f, optimize),
                None => eprintln!("Lütfen bir .ox dosyası belirtin."),
            }
        }
        "repl" => {
            start_repl();
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        filename if filename.ends_with(".ox") => {
            run_file(filename);
        }
        unknown => {
            eprintln!("Bilinmeyen komut: {}", unknown);
            print_help();
        }
    }
}

fn print_help() {
    println!(r#"
╔════════════════════════════════════════════════════════╗
║             OXIDE PROGRAMLAMA DİLİ                     ║
║       (Rust Tabanlı Sistem Dili Tasarımı)              ║
╚════════════════════════════════════════════════════════╝

Kullanım:
  oxide new <proje_adi>                    Yeni bir Oxide projesi şablonu oluşturur
  oxide clean                             'target/' derleme çıktılarını temizler
  oxide test [dosya.ox]                    Test fonksiyonlarını (test_*) otomatik çalıştırır
  oxide run [dosya.ox]                     Dosyayı (veya oxide.toml projesini) çalıştırır
  oxide check [dosya.ox]                   Derleme zamanı tip ve semantik denetimini yapar
  oxide build [dosya.ox] [seçenekler]      Kodu target/ içine C'ye çevirip native binary derler
  oxide tokens <dosya.ox>                  Kaynak kodun Lexer token akışını döker
  oxide ast <dosya.ox>                     Programın AST (Sözdizim Ağacı) çıktısını verir
  oxide hir <dosya.ox> [--opt]             Programın HIR (CFG & BasicBlocks) dökümünü verir
  oxide repl                               Etkileşimli REPL ortamını başlatır
  oxide help                               Bu yardım mesajını gösterir

Build Seçenekleri:
  -o <cikti>           Çıktı dosyasının adını / yolunu belirler
  --emit-c, -C         C derleyicisini çağırmaz, yalnızca üretilen .c dosyasını kaydeder
  --release            En yüksek optimizasyon seviyesini (-O3) ve release profilini kullanır
  -O0, -O1, -O2, -O3   Özel optimizasyon seviyesi belirler (varsayılan: -O2)
  --cc <derleyici>     Kullanılacak C derleyicisini seçer (varsayılan: gcc, ör. clang)
"#);
}

struct BuildConfig {
    file_path: String,
    output_path: Option<String>,
    emit_c: bool,
    opt_level: String,
    compiler: String,
    is_release: bool,
    project_info: Option<(std::path::PathBuf, project::PackageConfig)>,
}

fn parse_build_args(
    args: &[String],
    project_info: Option<(std::path::PathBuf, project::PackageConfig)>,
) -> Result<BuildConfig, String> {
    if args.is_empty() {
        return Err("Kullanım: oxide build [dosya.ox] [seçenekler]".to_string());
    }

    let mut file_path = None;
    let mut output_path = None;
    let mut emit_c = false;
    let mut opt_level = "-O2".to_string();
    let mut compiler = "gcc".to_string();
    let mut is_release = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                if i + 1 < args.len() {
                    output_path = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err("'-o' bayrağından sonra dosya yolu belirtilmelidir".to_string());
                }
            }
            "--emit-c" | "-C" => {
                emit_c = true;
            }
            "--release" => {
                is_release = true;
                opt_level = "-O3".to_string();
            }
            "-O0" | "-O1" | "-O2" | "-O3" | "-Os" => {
                opt_level = args[i].clone();
            }
            "--cc" => {
                if i + 1 < args.len() {
                    compiler = args[i + 1].clone();
                    i += 1;
                } else {
                    return Err("'--cc' bayrağından sonra derleyici adı belirtilmelidir (ör. gcc, clang)".to_string());
                }
            }
            arg if !arg.starts_with('-') && file_path.is_none() => {
                file_path = Some(arg.to_string());
            }
            unknown => {
                return Err(format!("Bilinmeyen build seçeneği: '{}'", unknown));
            }
        }
        i += 1;
    }

    let file_path = file_path.ok_or_else(|| "Derlenecek kaynak dosya (.ox) belirtilmedi".to_string())?;

    Ok(BuildConfig {
        file_path,
        output_path,
        emit_c,
        opt_level,
        compiler,
        is_release,
        project_info,
    })
}

fn check_file(path: &str) {
    let mut loader = ModuleLoader::new();
    let program = match loader.load_entry(Path::new(path)) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let program = match monomorph::Monomorphizer::monomorphize_program(program) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let program = match monomorph::Monomorphizer::monomorphize_program(program) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let mut checker = TypeChecker::new();
    if let Err(err) = checker.check_program(&program) {
        eprintln!("{}", err);
        return;
    }

    let hir_prog = HirBuilder::lower_program(&program);
    if let Err(err) = BorrowChecker::check_program(&hir_prog) {
        eprintln!("{}", err);
        return;
    }

    println!("✔ [Başarılı] Derleme zamanı tip, semantik ve borrow denetimi geçti. (0 hata)");
}

fn dump_tokens(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Dosya okunamadı '{}': {}", path, e);
            return;
        }
    };

    let mut lexer = Lexer::new(&source);
    match lexer.tokenize() {
        Ok(tokens) => {
            println!("=== Token Akışı: '{}' (Toplam: {}) ===", path, tokens.len());
            for (idx, tok) in tokens.iter().enumerate() {
                println!("{:4}: [{}:{:2}] {:?}", idx + 1, tok.span.line, tok.span.col, tok.kind);
            }
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}

fn build_file(config: BuildConfig) {
    let path = &config.file_path;
    let mut loader = ModuleLoader::new();
    let program = match loader.load_entry(Path::new(path)) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let mut program = match monomorph::Monomorphizer::monomorphize_program(program) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    // Compile-time static semantic & type checking
    let mut checker = TypeChecker::new();
    if let Err(err) = checker.check_program(&program) {
        eprintln!("{}", err);
        return;
    }

    let hir_prog = HirBuilder::lower_program(&program);
    if let Err(err) = BorrowChecker::check_program(&hir_prog) {
        eprintln!("{}", err);
        return;
    }

    // AST Constant Folding & Dead Code Optimization
    opt::optimize_ast_program(&mut program);

    let codegen = CodegenC::new();
    let c_code = codegen.compile_program(&program);

    let profile = if config.is_release { "release" } else { "debug" };

    let (target_root, app_name) = match &config.project_info {
        Some((toml_path, pkg_cfg)) => {
            let root = toml_path.parent().unwrap_or(Path::new("."));
            let _ = project::update_lockfile(root, pkg_cfg);
            (root.join("target"), pkg_cfg.name.clone())
        }
        None => {
            let stem = Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("app")
                .to_string();
            (std::path::PathBuf::from("target"), stem)
        }
    };

    let target_profile_dir = target_root.join(profile);
    let c_gen_dir = target_profile_dir.join("c_gen");
    let _ = fs::create_dir_all(&c_gen_dir);
    project::ensure_target_gitignore(&target_root);

    let (final_binary, c_file) = match &config.output_path {
        Some(custom_out) => {
            if let Some(parent) = Path::new(custom_out).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = fs::create_dir_all(parent);
                }
            }
            if config.emit_c {
                let c_path = if custom_out.ends_with(".c") {
                    custom_out.clone()
                } else {
                    format!("{}.c", custom_out)
                };
                (custom_out.clone(), c_path)
            } else {
                let c_path = c_gen_dir.join(format!("{}.c", app_name)).to_string_lossy().to_string();
                (custom_out.clone(), c_path)
            }
        }
        None => {
            let bin_path = target_profile_dir.join(&app_name).to_string_lossy().to_string();
            let c_path = c_gen_dir.join(format!("{}.c", app_name)).to_string_lossy().to_string();
            (bin_path, c_path)
        }
    };

    if let Err(e) = fs::write(&c_file, &c_code) {
        eprintln!("C dosyası yazılamadı: {}", e);
        return;
    }

    if config.emit_c {
        println!("✔ C kaynak kodu diske yazıldı -> {}", c_file);
        return;
    }

    println!("⚡ Oxide -> C üretildi: {}", c_file);
    println!("🔨 {} ile native binary derleniyor (Profil: {}, Optimizasyon: {})...", config.compiler, profile, config.opt_level);

    let cc_status = Command::new(&config.compiler)
        .args(&[&config.opt_level, &c_file, "-o", &final_binary, "-lm"])
        .status();

    match cc_status {
        Ok(status) if status.success() => {
            println!("✔ Başarıyla derlendi [{}] -> {} (Native Binary)", config.compiler, final_binary);
            println!("Çalıştırmak için: ./{}", final_binary);
        }
        Ok(status) => {
            eprintln!("{} derleme hatası kodu: {}", config.compiler, status);
        }
        Err(e) => {
            eprintln!("'{}' çalıştırılamadı: {}. Sisteminizde kurulu olduğundan emin olun.", config.compiler, e);
        }
    }
}

fn run_file(path: &str) {
    let mut loader = ModuleLoader::new();
    let program = match loader.load_entry(Path::new(path)) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let mut program = match monomorph::Monomorphizer::monomorphize_program(program) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    // Compile-time static semantic & type checking
    let mut checker = TypeChecker::new();
    if let Err(err) = checker.check_program(&program) {
        eprintln!("{}", err);
        return;
    }

    let hir_prog = HirBuilder::lower_program(&program);
    if let Err(err) = BorrowChecker::check_program(&hir_prog) {
        eprintln!("{}", err);
        return;
    }

    // AST Constant Folding & Dead Code Optimization
    opt::optimize_ast_program(&mut program);

    let mut interpreter = Interpreter::new();
    interpreter.load_program(program);

    if let Err(err) = interpreter.run_main() {
        eprintln!("{}", err);
    }
}

fn dump_ast(path: &str) {
    let mut loader = ModuleLoader::new();
    match loader.load_entry(Path::new(path)) {
        Ok(p) => {
            println!("{:#?}", p);
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}

fn dump_hir(path: &str, optimize: bool) {
    let mut loader = ModuleLoader::new();
    let program = match loader.load_entry(Path::new(path)) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let program = match monomorph::Monomorphizer::monomorphize_program(program) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let mut typechecker = TypeChecker::new();
    if let Err(err) = typechecker.check_program(&program) {
        eprintln!("{}", err);
        return;
    }

    let mut hir_prog = HirBuilder::lower_program(&program);
    if optimize {
        opt::optimize_hir_program(&mut hir_prog);
    }
    println!("{}", hir_prog);
}

fn start_repl() {
    println!("Oxide v0.1.0 REPL");
    println!("Çıkmak için 'exit' veya 'quit' yazın.\n");

    let mut interpreter = Interpreter::new();
    let stdin = io::stdin();

    loop {
        print!("ox> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() || line.trim().is_empty() {
            continue;
        }

        let trimmed = line.trim();
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }

        // Try as an expression first
        let mut lexer = Lexer::new(trimmed);
        if let Ok(tokens) = lexer.tokenize() {
            let mut parser = Parser::new(tokens);
            if let Ok(expr) = parser.parse_expression(0) {
                match interpreter.eval_expr(&expr) {
                    Ok(val) => {
                        println!("=> {}", val);
                        continue;
                    }
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                }
            }
        }

        // Try parsing as block / function definition
        let wrapped = format!("fn main() {{ {} }}", trimmed);
        let mut lexer = Lexer::new(&wrapped);
        match lexer.tokenize() {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                match parser.parse_program() {
                    Ok(program) => {
                        interpreter.load_program(program);
                        if let Err(e) = interpreter.run_main() {
                            println!("{}", e);
                        }
                    }
                    Err(e) => println!("Hata: {}", e),
                }
            }
            Err(e) => println!("Sözcük Hatası: {}", e),
        }
    }
}
