# Claude Asistan Rehberi (claude.md)

Bu dosya, Oxide projesi üzerinde çalışırken yapay zeka asistanları (Claude vb.) için mimari rehberi, komutları ve kodlama kurallarını tanımlar.

---

## 📌 Proje Özeti
- **Adı:** Oxide (`.ox`)
- **Tanım:** Rust dili ile geliştirilmiş, Rust sözdiziminden ve bellek güvenliği felsefesinden ilham alan modern bir programlama dili ve yorumlayıcısı/derleyicisi.
- **Teknoloji:** Rust (2024 Edition), harici crate bağımlılığı bulunmamaktadır (Zero external dependencies).

---

## 🛠️ Temel Komutlar

### Derleme ve Kontrol
```bash
cargo check                # Hızlı sözdizim ve tip kontrolü
cargo build                # Debug derlemesi
cargo build --release      # Optimize release derlemesi
cargo test                 # Birim ve entegrasyon testleri
```

### Çalıştırma Komutları
```bash
cargo run -- run <dosya.ox>                     # Oxide programını tip denetimi yapıp yorumlayarak çalıştırır
cargo run -- check <dosya.ox>                   # Derleme öncesi statik tip ve semantik denetimi yapar
cargo run -- build <dosya.ox> [seçenekler]      # Kodu C'ye transpiled edip native binary derler
                                                # Seçenekler: -o <ad>, --emit-c, --release, -O3, --cc clang
cargo run -- tokens <dosya.ox>                  # Yalnızca Lexer token akışını satır/sütun dökümüyle basar
cargo run -- ast <dosya.ox>                     # Programın AST çıktısını döker
cargo run -- repl                               # Etkileşimli REPL ortamını açar
```

---

## 🧱 Mimari Yapı

Proje modüler bir derleyici boru hattı (pipeline) şeklinde yapılandırılmıştır:

```
src/
├── span.rs      # Kaynak kod satır/sütun koordinatları (Span)
├── token.rs     # TokenKind ve Token veri yapıları
├── lexer.rs     # Lexer (Tokenizer) - Kaynak metni tokenlara ayırır
├── ast.rs       # Abstract Syntax Tree (Expr, Stmt, Block, Function, Program)
├── parser.rs    # Pratt Parser tabanlı Recursive Descent sözdizim çözümleyici
├── typecheck.rs # Statik tip ve semantik denetleyici (Compile-time TypeChecker)
├── eval.rs      # Çevre (Environment), Değerler (Value), Değişmezlik denetimi ve Yorumlayıcı
├── codegen_c.rs # AST -> C11 Transpiler ve Native derleme üreticisi
└── main.rs      # CLI argüman yönetimi, build/run/repl koordinasyonu
```

---

## 📐 Kodlama Standartları ve İlkeleri

1. **Sıfır Dış Bağımlılık Prensibi:**
   - Zorunlu olmadıkça `Cargo.toml` içerisine harici crate ekleme. Standart kütüphanenin (`std`) gücünü kullan.
2. **Rust-First İfade Mantığı:**
   - Bloklar ve if yapıları Rust gibi değer üretebilir (expression-oriented). Son ifadenin noktalı virgülsüz olması kuralını koru.
3. **Anlaşılır Hata Raporlama:**
   - Hata mesajlarında daima `Hata [satır:sütun]: ...` formatını kullan.
   - Değiştirilemez değişkenlere atama yapılmaya çalışıldığında kullanıcıya `let mut` önerecek açıklayıcı mesajlar ver.
4. **Kod Temizliği:**
   - Kodları modüllere ayrılmış, test edilebilir ve dokümante edilmiş tut.
   - `cargo check` çıktısında gereksiz `warning` bırakma.
