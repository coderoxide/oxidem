# Oxide (`.ox`) 🦀

Oxide, Rust dili ile geliştirilmiş, Rust sözdiziminden ve bellek güvenliği felsefesinden ilham alan modern bir programlama dilidir. Hem doğrudan AST tabanlı hızlı bir **yorumlayıcıya (Interpreter)** hem de C11 üzerinden GCC ile çalışan **Native Makine Kodu Derleyicisine (Native Compiler)** sahiptir.

---

## 🚀 Temel Özellikler

- **Rust Benzeri Sözdizimi:** `fn`, `let`, `let mut`, `if / else`, `while`, `return`.
- **İfade Odaklı (Expression-Oriented):** Blokların (`{ ... }`), `if/else` ve `match` yapılarının son satırında noktalı virgül konulmadığında otomatik olarak değer döner.
- **Değişmezlik (Immutability by Default):** `let x = 5;` ile tanımlanan değişkenler sabittir. Değiştirmek için `let mut x = 5;` yazılmalıdır.
- **Veri Yapıları:** Dinamik Diziler (`[T]`), `struct` tanımları ve davranış kazandıran `impl` blokları (`self` metotları).
- **Desen Eşleştirme (Pattern Matching):** `enum` tipleri, `match` ifadeleri, `_` wildcard ve değişken bağlama (variable binding).
- **Çift Yürütme Motoru:** Hem anında yorumlama (`oxide run`) hem de GCC `-O2` ile native ELF 64-bit binary derleme (`oxide build`).
- **Sıfır Dış Bağımlılık (Zero External Dependencies):** Yalnızca saf Rust standart kütüphanesi.

---

## 📖 Kapsamlı Dokümantasyon (`docs/`)

Oxide projesi için hazırlanan detaylı teknik rehberler:

* [📚 Dokümantasyon İndeksi (docs/README.md)](./docs/README.md)
* [📝 Sözdizimi Rehberi (docs/syntax_guide.md)](./docs/syntax_guide.md)
* [📦 Veri Yapıları: Diziler, Struct ve Impl (docs/data_structures.md)](./docs/data_structures.md)
* [🎯 Desen Eşleştirme ve Enum'lar (docs/pattern_matching.md)](./docs/pattern_matching.md)
* [⚙️ Derleyici ve Transpiler Mimarisi (docs/compiler_architecture.md)](./docs/compiler_architecture.md)
* [💻 CLI ve Araç Takımı Referansı (docs/cli_reference.md)](./docs/cli_reference.md)
* [💡 Tasarım Felsefesi ve SSS (docs/faq_and_design_philosophy.md)](./docs/faq_and_design_philosophy.md)

---

## 🛠️ Nasıl Çalıştırılır?

### 1. Yorumlayarak Çalıştırma (Interpreter)
```bash
cargo run -- run examples/hello.ox
cargo run -- run examples/fibonacci.ox
cargo run -- run examples/structs.ox
cargo run -- run examples/enums_and_match.ox
```

### 2. Native Makine Koduna Derleme (Native Binary Compiler)
```bash
cargo run -- build examples/fibonacci.ox
./examples/fibonacci
```

### 3. AST (Sözdizim Ağacı) İnceleme
```bash
cargo run -- ast examples/hello.ox
```

### 4. Etkileşimli Konsol (REPL)
```bash
cargo run -- repl
```

---

## 📝 Örnek Kod

```rust
struct Player {
    name: str,
    score: int,
}

impl Player {
    fn add_score(self, pts: int) {
        self.score = self.score + pts;
    }
}

enum Status {
    Active,
    Idle,
}

fn main() {
    let mut p = Player { name: "Ahmet", score: 100 };
    p.add_score(50);
    println(p.name + " puanı:", p.score);

    let durum = Status::Active;
    let mesaj = match durum {
        Status::Active => "Oyuncu aktif!",
        Status::Idle   => "Oyuncu beklemede.",
    };
    println(mesaj);
}
```
# oxidem
