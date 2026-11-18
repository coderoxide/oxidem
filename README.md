# Oxide (`.ox`) 🦀 v1.0.0

[![Release](https://img.shields.io/badge/version-1.0.0-brightgreen.svg)](https://github.com/coderoxide/oxidem)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Dependencies](https://img.shields.io/badge/dependencies-0%20(Pure%20Rust%20std)-orange.svg)](Cargo.toml)
[![Tests](https://img.shields.io/badge/tests-36%2F36%20passing%20(100%25)-success.svg)](tests/)

**Oxide**, sıfırdan ve harici hiçbir kütüphane/crate kullanılmadan (**0 external dependencies, 100% pure Rust `std`**) geliştirilmiş modern, güvenli ve yüksek başarımlı bir sistem programlama dilidir.

Oxide, hem doğrudan sözdizim ağacı üzerinden çalışan yüksek hızlı bir **yorumlayıcıya (Interpreter)** hem de C11 üzerinden POSIX sistem çağrılarıyla çalışan ve GCC `-O2` optimizasyonuyla doğrudan doğal makine koduna derleyen bir **Native Derleyiciye (Compiler)** sahiptir. Yorumlayıcı (`oxide run`) ve GCC native binary (`oxide build`) arasında **%100 çıktı ve davranış uyumu (parity)** garanti edilir.

---

## 🌟 Öne Çıkan Yetenekler

* **Rust İlhamlı Güçlü Sözdizimi:** `fn`, `let`, `let mut`, `if / else`, `while`, `for .. in`, `break`, `continue`, `return`.
* **İfade Odaklı Tasarım (Expression-Oriented):** Bloklar `{ ... }`, `if/else` ve `match` ifadeleri son satırda noktalı virgül (`- ;`) konulmadığında doğrudan değer döndürür.
* **Derleme Zamanı Güvenliği & Borrow Checker:** Sahiplik (Ownership), Ödünç Alma (`&T`, `&mut T`), Sözlüksel Olmayan Yaşam Süreleri (NLL) ve `unsafe` blok denetimi.
* **Gelişmiş Tip Sistemi & Generics:**
  * Jenerik fonksiyonlar, struct'lar ve impl blokları (`Monomorphization` ile sıfır ek yük).
  * Trait ve arayüz sistemi (Trait bounds: `T: Display + Clone`).
  * `Option<T>` ve `Result<T, E>` hata yönetimi ve Rust tarzı `?` (Try) operatörü.
* **Akıllı İşaretçiler (Smart Pointers):** `Box<T>`, `Rc<T>`, `RefCell<T>` ile özyinelemeli veri yapıları (bağlı listeler, ikili ağaçlar).
* **Eşzamanlılık & Çoklu İş Parçacığı (Concurrency):** `thread::spawn`, `thread::sleep`, `channel` (Sender/Receiver mesaj kuyruğu) ve `Mutex<T>` ile güvenli paylaşımlı bellek.
* **Ağ & Soket Programlama (POSIX Sockets):** `TcpListener` ve `TcpStream` ile tam teşekküllü TCP istemci/sunucu mimarisi.
* **HTTP Sunucusu, REST API & JSON:** `http::get`, `http::post`, `http::response`, `json::stringify`, `json::parse`, `json::get`.
* **Kriptografi, Base64 & Hassas Zamanlama:** 64-bit FNV-1a özetleme (`crypto::hash`), IEEE 802.3 CRC-32 checksum (`crypto::crc32`), RFC 4648 `base64::encode/decode`, Unix epoch ve monotonik kronometre (`Instant::now`).
* **Geliştirici Araçları (Dev Tools):**
  * `///` dokümantasyon yorumlarından otomatik Markdown ve çevrimdışı HTML5 API belgesi üretimi (`oxide doc`).
  * Kod standartlaştırıcı ve CI/CD stil denetleyicisi (`oxide fmt`, `oxide fmt --check`).
* **Proje ve Paket Yönetimi:** `oxide new`, `oxide build`, `oxide test`, `oxide clean`, `oxide.toml` ve deterministik bağımlılık kilit dosyası `oxide.lock`.

---

## 🚀 Hızlı Başlangıç & Kurulum

### Kurulum (Cargo ile Sisteme Ekleme)
```bash
git clone https://github.com/coderoxide/oxidem.git
cd oxidem
cargo install --path .
```

### Temel CLI Komutları
```bash
# Sürüm kontrolü
oxide version

# Kaynak kodunu doğrudan yorumlayıcı ile çalıştırma
oxide run examples/hello.ox

# Statik tip ve semantik denetimi yapma (derlemeden hata yakalama)
oxide check examples/hello.ox

# Kodu C'ye çevirip GCC -O2 ile optimize ELF native binary derleme
oxide build examples/hello.ox -o hello_bin
./hello_bin

# Dokümantasyon üretme (Markdown ve HTML5)
oxide doc examples/doc_and_fmt_demo.ox -o docs/

# Kaynak kodunu standart Oxide formatına dönüştürme
oxide fmt examples/doc_and_fmt_demo.ox
oxide fmt examples/doc_and_fmt_demo.ox --check

# Test motorunu çalıştırma
oxide test
```

---

## 📝 Örnek Kod

```rust
/// İki boyutlu uzayda bir noktayı temsil eder.
struct Point {
    x: int,
    y: int,
}

impl Point {
    fn info(self) -> str {
        f"Point({self.x}, {self.y})"
    }
}

fn topla(a: int, b: int) -> int {
    a + b
}

fn main() {
    let p = Point { x: 10, y: 20 };
    println("Nokta:", p.info());

    let toplam = topla(15, 27);
    println("Toplam:", toplam);
    assert(toplam == 42);

    let kronometre = Instant::now();
    let hash = crypto::hash("Oxide Systems Language");
    let b64 = base64::encode("Gizli Mesaj");

    println("Hash:", hash);
    println("Base64:", b64);
    println("Geçen süre (us):", kronometre.elapsed_micros());
}
```

---

## 📖 Kapsamlı Dokümantasyon (`docs/`)

* [📚 Dokümantasyon İndeksi (docs/README.md)](./docs/README.md)
* [📝 Sözdizimi Rehberi (docs/syntax_guide.md)](./docs/syntax_guide.md)
* [📦 Veri Yapıları: Diziler, Struct ve Impl (docs/data_structures.md)](./docs/data_structures.md)
* [🎯 Desen Eşleştirme ve Enum'lar (docs/pattern_matching.md)](./docs/pattern_matching.md)
* [⚙️ Derleyici ve Transpiler Mimarisi (docs/compiler_architecture.md)](./docs/compiler_architecture.md)
* [💻 CLI ve Araç Takımı Referansı (docs/cli_reference.md)](./docs/cli_reference.md)
* [💡 Tasarım Felsefesi ve SSS (docs/faq_and_design_philosophy.md)](./docs/faq_and_design_philosophy.md)

---

## 📄 Lisans

Bu proje MIT veya Apache-2.0 lisansı altında lisanslanmıştır.
