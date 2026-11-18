use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct PluginManager;

impl PluginManager {
    /// Handles the `oxide plugin` CLI subcommand.
    pub fn handle_cli(args: &[String]) {
        if args.is_empty() {
            Self::print_help();
            return;
        }

        match args[0].as_str() {
            "new" => {
                if args.len() < 2 {
                    eprintln!("Kullanım: oxide plugin new <eklenti_adi> [--type rust|c]");
                    return;
                }
                let name = &args[1];
                let mut plugin_type = "rust";
                let mut i = 2;
                while i < args.len() {
                    if args[i] == "--type" && i + 1 < args.len() {
                        plugin_type = &args[i + 1];
                        i += 1;
                    }
                    i += 1;
                }

                if let Err(e) = Self::init_new_plugin(name, plugin_type) {
                    eprintln!("Hata: Eklenti oluşturulamadı: {}", e);
                }
            }
            "build" => {
                let target = args.get(1).map(|s| s.as_str());
                if let Err(e) = Self::build_plugin(target) {
                    eprintln!("Hata: Eklenti derlenemedi: {}", e);
                }
            }
            "list" => {
                Self::list_plugins();
            }
            "help" | "--help" | "-h" => {
                Self::print_help();
            }
            unknown => {
                eprintln!("Bilinmeyen plugin komutu: '{}'", unknown);
                Self::print_help();
            }
        }
    }

    fn print_help() {
        println!(r#"
Oxide Eklenti ve Yerel Kütüphane Yöneticisi

Kullanım:
  oxide plugin new <ad> [--type rust|c]    Yeni bir Rust veya C yerel eklenti şablonu oluşturur
  oxide plugin build [eklenti_yolu]       Eklentiyi derleyip paylaşımlı kütüphane (.so) üretir
  oxide plugin list                       Mevcut eklentileri listeler
  oxide plugin help                       Bu yardım mesajını gösterir
"#);
    }

    /// Initializes a new native plugin scaffold under `plugins/<name>`.
    pub fn init_new_plugin(name: &str, plugin_type: &str) -> Result<(), String> {
        let clean_name = name.trim().replace('-', "_");
        let plugin_dir = if name.contains('/') || name.contains('\\') {
            PathBuf::from(name)
        } else {
            Path::new("plugins").join(&clean_name)
        };

        if plugin_dir.exists() {
            return Err(format!("'{}' dizini zaten mevcut!", plugin_dir.display()));
        }

        fs::create_dir_all(plugin_dir.join("src"))
            .map_err(|e| format!("Dizin oluşturulamadı: {}", e))?;

        match plugin_type.to_lowercase().as_str() {
            "rust" | "rs" => {
                // Cargo.toml for cdylib
                let cargo_toml = format!(
                    r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "staticlib"]

[dependencies]
"#,
                    name = clean_name
                );
                fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)
                    .map_err(|e| format!("Cargo.toml yazılamadı: {}", e))?;

                // src/lib.rs
                let lib_rs = format!(
                    r#"// Oxide Yerel Rust Eklentisi: {name}
// Bu kütüphane Oxide içerisinden sıfır maliyetli C-ABI ile çağrılır.

#[unsafe(no_mangle)]
pub extern "C" fn {name}_selam() -> i64 {{
    println!("[Oxide Rust Plugin: {name}] Eklenti başarıyla aktif edildi!");
    1
}}

#[unsafe(no_mangle)]
pub extern "C" fn {name}_topla(a: i64, b: i64) -> i64 {{
    a + b
}}

#[unsafe(no_mangle)]
pub extern "C" fn {name}_mesafe_kare(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {{
    let dx = x2 - x1;
    let dy = y2 - y1;
    dx * dx + dy * dy
}}
"#,
                    name = clean_name
                );
                fs::write(plugin_dir.join("src/lib.rs"), lib_rs)
                    .map_err(|e| format!("src/lib.rs yazılamadı: {}", e))?;

                // interface.ox
                let interface_ox = format!(
                    r#"// Oxide Arayüz Başlığı: {name}
// Kullanım: import "plugins/{name}/interface.ox";

extern "C" {{
    fn {name}_selam() -> int;
    fn {name}_topla(a: int, b: int) -> int;
    fn {name}_mesafe_kare(x1: float, y1: float, x2: float, y2: float) -> float;
}}
"#,
                    name = clean_name
                );
                fs::write(plugin_dir.join("interface.ox"), interface_ox)
                    .map_err(|e| format!("interface.ox yazılamadı: {}", e))?;
            }
            "c" => {
                // src/lib.c
                let lib_c = format!(
                    r#"// Oxide Yerel C Eklentisi: {name}
#include <stdio.h>
#include <stdint.h>

int64_t {name}_selam(void) {{
    printf("[Oxide C Plugin: {name}] Eklenti başarıyla aktif edildi!\n");
    return 1;
}}

int64_t {name}_topla(int64_t a, int64_t b) {{
    return a + b;
}}
"#,
                    name = clean_name
                );
                fs::write(plugin_dir.join("src/lib.c"), lib_c)
                    .map_err(|e| format!("src/lib.c yazılamadı: {}", e))?;

                // interface.ox
                let interface_ox = format!(
                    r#"// Oxide Arayüz Başlığı: {name}

extern "C" {{
    fn {name}_selam() -> int;
    fn {name}_topla(a: int, b: int) -> int;
}}
"#,
                    name = clean_name
                );
                fs::write(plugin_dir.join("interface.ox"), interface_ox)
                    .map_err(|e| format!("interface.ox yazılamadı: {}", e))?;
            }
            other => return Err(format!("Desteklenmeyen eklenti tipi: '{}' (geçerli: rust, c)", other)),
        }

        println!("✔ [Eklenti] '{}' ({}) şablonu başarıyla oluşturuldu -> {}", clean_name, plugin_type, plugin_dir.display());
        println!("  Derlemek için: oxide plugin build {}", plugin_dir.display());
        Ok(())
    }

    /// Builds a plugin in the specified directory or all plugins in `plugins/`.
    pub fn build_plugin(target: Option<&str>) -> Result<(), String> {
        let targets = if let Some(t) = target {
            vec![PathBuf::from(t)]
        } else if Path::new("plugins").is_dir() {
            let mut list = Vec::new();
            if let Ok(entries) = fs::read_dir("plugins") {
                for e in entries.flatten() {
                    if e.path().is_dir() {
                        list.push(e.path());
                    }
                }
            }
            list
        } else if Path::new("Cargo.toml").exists() {
            vec![PathBuf::from(".")]
        } else {
            return Err("Derlenecek eklenti dizini bulunamadı ('plugins/' klasörü veya dosya yolu belirtilmedi)".to_string());
        };

        if targets.is_empty() {
            return Err("Derlenecek eklenti bulunamadı.".to_string());
        }

        for dir in targets {
            let name = dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("eklenti");

            let cargo_toml = dir.join("Cargo.toml");
            let c_src = dir.join("src/lib.c");

            if cargo_toml.exists() {
                println!("🔨 [Rust Eklentisi] '{}' derleniyor (cargo build --release)...", name);
                let status = Command::new("cargo")
                    .args(["build", "--release", "--manifest-path", cargo_toml.to_str().unwrap()])
                    .status()
                    .map_err(|e| format!("cargo çalıştırılamadı: {}", e))?;

                if !status.success() {
                    return Err(format!("'{}' derlemesi başarısız oldu", name));
                }

                // Locate resulting .so / .a in target/release
                let release_dir = dir.join("target/release");
                let so_name = format!("lib{}.so", name);
                let a_name = format!("lib{}.a", name);

                if release_dir.join(&so_name).exists() {
                    let _ = fs::copy(release_dir.join(&so_name), dir.join(&so_name));
                    println!("✔ [Eklenti] Paylaşımlı kütüphane hazır: {}/{}", dir.display(), so_name);
                } else if release_dir.join(&a_name).exists() {
                    let _ = fs::copy(release_dir.join(&a_name), dir.join(&a_name));
                    println!("✔ [Eklenti] Statik kütüphane hazır: {}/{}", dir.display(), a_name);
                }
            } else if c_src.exists() {
                println!("🔨 [C Eklentisi] '{}' derleniyor (gcc -shared -fPIC)...", name);
                let so_path = dir.join(format!("lib{}.so", name));
                let status = Command::new("gcc")
                    .args([
                        "-shared",
                        "-fPIC",
                        "-O2",
                        c_src.to_str().unwrap(),
                        "-o",
                        so_path.to_str().unwrap(),
                    ])
                    .status()
                    .map_err(|e| format!("gcc çalıştırılamadı: {}", e))?;

                if !status.success() {
                    return Err(format!("'{}' C derlemesi başarısız oldu", name));
                }
                println!("✔ [Eklenti] Paylaşımlı kütüphane hazır: {}", so_path.display());
            } else {
                println!("Uyarı: '{}' içinde derlenebilir dosya (Cargo.toml veya src/lib.c) bulunamadı.", dir.display());
            }
        }

        Ok(())
    }

    /// Lists all discovered native plugins in `plugins/`.
    pub fn list_plugins() {
        let plugins_dir = Path::new("plugins");
        if !plugins_dir.is_dir() {
            println!("Henüz tanımlanmış bir eklenti yok ('plugins/' dizini mevcut değil).");
            println!("Yeni bir eklenti oluşturmak için: oxide plugin new <eklenti_adi>");
            return;
        }

        println!("📦 Mevcut Oxide Eklentileri:");
        if let Ok(entries) = fs::read_dir(plugins_dir) {
            let mut count = 0;
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("?");
                    let is_rust = p.join("Cargo.toml").exists();
                    let is_c = p.join("src/lib.c").exists();
                    let e_type = if is_rust { "Rust (cdylib)" } else if is_c { "C (shared lib)" } else { "Bilinmiyor" };
                    let has_so = p.join(format!("lib{}.so", name)).exists();
                    let status = if has_so { "✔ derlenmiş" } else { "⏳ derleme bekliyor" };
                    println!("  • {} [{}] - {}", name, e_type, status);
                    count += 1;
                }
            }
            if count == 0 {
                println!("  (Henüz eklenti eklenmemiş)");
            }
        }
    }
}
