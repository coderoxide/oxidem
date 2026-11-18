# Oxide Dil Dokümantasyonu (`docs/`) 🦀

Oxide programlama dili için kapsamlı teknik dokümantasyon merkezine hoş geldiniz.

Bu rehber, Oxide dilinin sözdiziminden tip sistemine, derleyici mimarisinden native C derleme sürecine kadar tüm ayrıntıları içerir.

---

## 📚 Dokümantasyon İndeksi

| Doküman | Açıklama |
| :--- | :--- |
| [1. Sözdizimi Rehberi (syntax_guide.md)](./syntax_guide.md) | Değişkenler, Değişmezlik (`let` vs `let mut`), İfade Odaklı Bloklar, Fonksiyonlar ve Kontrol Akışı. |
| [2. Veri Yapıları Rehberi (data_structures.md)](./data_structures.md) | Dinamik Diziler (`[T]`), `struct` tanımları, `impl` blokları ve `self` metot mekanizması. |
| [3. Desen Eşleştirme Rehberi (pattern_matching.md)](./pattern_matching.md) | `enum` tipleri, `match` ifadeleri, değişken bağlama (binding) ve wildcard (`_`). |
| [4. Derleyici Mimarisi (compiler_architecture.md)](./compiler_architecture.md) | Lexer, Pratt Parser, AST, Kapsam Denetimli Yorumlayıcı ve C11 Transpiler boru hattı. |
| [5. CLI ve Araç Takımı (cli_reference.md)](./cli_reference.md) | `run`, `build`, `ast`, `repl` komutlarının kullanımı ve bayrakları. |
| [6. Tasarım Felsefesi ve SSS (faq_and_design_philosophy.md)](./faq_and_design_philosophy.md) | Neden Oxide? Rust benzerliği, bellek modeli, C derleme avantajları ve yol haritası. |

---

## ⚡ Hızlı Başlangıç (Quickstart)

### Kurulum ve Derleme
Oxide, saf Rust (Zero external dependencies) ile yazılmıştır. Çalıştırmak için yalnızca standart Rust aracına (`cargo`) ve native derleme için `gcc`'ye ihtiyaç duyar.

```bash
# Projeyi derleyin
cargo build --release

# Bir Oxide dosyasını anında yorumlayarak çalıştırın
cargo run -- run examples/hello.ox

# Doğrudan native makine koduna (ELF binary) derleyin
cargo run -- build examples/fibonacci.ox
./examples/fibonacci

# Etkileşimli REPL konsolunu açın
cargo run -- repl
```
