# Oxide Dil Geliştirme İlerleme Takibi (progress.md)

Bu dosya, Rust tabanlı Oxide (`.ox`) programlama dili ve derleyici/yorumlayıcı projesinin mevcut durumunu, tamamlanan özellikleri ve gelecek hedeflerini içerir.

---

## 📊 Genel Durum
- **Proje Adı:** Oxide (`.ox`)
- **Geliştirme Dili:** Rust (2024 edition)
- **Mevcut Sürüm:** v0.18.0 (Ağ ve Soket Programlama TcpListener/TcpStream, Eşzamanlılık ve Çoklu İş Parçacığı thread::spawn/sleep, Mesajlaşma Kanalları channel/send/recv, Karşılıklı Dışlama Mutex<T>, Derleme Zamanı Sabitleri const, Standart Dosya Sistemi fs::, Sistem ve Çevre Değişkenleri env::, Akıllı İşaretçiler Box/Rc/RefCell, Target Build Dizin Mimarisi, oxide.lock Kilit Dosyası, oxide clean, Operator Overloading & Traits, Slices & Range Indexing, String Interpolation f"...", Tuples & Destructuring, Trait & Interface System, Option/Result & Try '?' Operator, Generics & Monomorphization, Closures, Arena Management, HIR Optimizer, Test Engine & oxide.toml)
- **Bağımlılık Durumu:** Sıfır dış bağımlılık (Pure Rust Standard Library)
- **Çalışma Modları:** Yorumlayıcı (`run`), Tip Denetleyici (`check`), Native GCC Derleyici (`build`), Token Dökücü (`tokens`), AST Dökücü (`ast`), HIR Dökücü (`hir`), REPL (`repl`)

---

## ✅ Tamamlanan Özellikler (v0.1.0)

### 1. Sözcüksel Analiz (Lexer)
- [x] Satır ve sütun bilgisi tutan kaynak konumu (`Span`).
- [x] Sayı literalleri (`i64` tamsayılar ve `f64` ondalıklı sayılar).
- [x] Metin dizgileri (`str`) ve kaçış karakterleri (`\n`, `\t`, `\"`, `\\`).
- [x] Boolean literalleri (`true`, `false`).
- [x] Anahtar sözcükler (`fn`, `let`, `mut`, `if`, `else`, `while`, `return`).
- [x] Temel tip sözcükleri (`int`, `float`, `bool`, `str`, `void`).
- [x] Aritmetik ve mantıksal operatörler (`+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`, `!`).
- [x] Çift karakterli token ve tek satırlı yorum (`//`) desteği.

### 2. Sözdizim Ayrıştırıcı (Parser & AST)
- [x] Öncelik tırmanışlı **Pratt Parser** ile operatör önceliklerinin eksiksiz çözümlenmesi.
- [x] Rust tarzı **İfade Odaklı (Expression-Oriented)** blok yapısı (`{ ... }` bloğunun son ifadesi noktalı virgülsüz değer döner).
- [x] `fn name(param: type) -> type { ... }` fonksiyon bildirimleri.
- [x] Değişken tanımlamaları (`let x = ...;`, `let mut x: int = ...;`).
- [x] Kontrol akışı: `if` / `else if` / `else` ifadeleri ve `while` döngüsü.
- [x] `return` deyimi.

### 3. Çalışma Zamanı & Yorumlayıcı (Evaluator)
- [x] Hiyerarşik kapsam (Lexical Scoping & Environment) yönetimi.
- [x] **Rust Tarzı Değişmezlik (Immutability Check):** `let` ile tanımlanan değişkenlerin, dizi elemanlarının ve struct alanlarının yeniden atanmasını engelleyen ve anlamlı hata mesajı veren mekanizma.
- [x] Özyinelemeli (Recursive) fonksiyon çağrıları (Fibonacci vb.).
- [x] Dahili fonksiyonlar: `println(...)`, `print(...)`, `assert(...)`, `len(...)`, `push(...)`, `pop(...)`.
- [x] Tip dönüşümleri (metin birleştirme vb. operatör kolaylıkları).
- [x] **Dinamik Diziler (Arrays):** `[1, 2, 3]` literali, `arr[i]` indeks erişimi ve `arr[i] = x` indeks ataması.
- [x] **Struct & Impl Mimarisi:** Özel veri tipleri, `Point { x: 1, y: 2 }` başlatıcıları, nokta operatörü ile alan erişimi (`p.x`), alan ataması (`p.x = 10`) ve `self` içeren metot çağrıları (`p.info()`).
- [x] **Enum & Pattern Matching (`match`):**
  - `enum` tanımları ve `Enum::Variant` yol çözümlemesi (`::` operatörü).
  - İfade odaklı `match` yapısı (Rust'taki gibi değer dönebilir).
  - Değişken bağlama (`val => ...`), Wildcard (`_`) ve varyant eşleştirme desteği.

### 4. Geliştirici Deneyimi (CLI & REPL & Compiler)
- [x] `oxide run <dosya.ox>`: Kaynak dosyasını tip denetiminden geçirip çalıştırır.
- [x] `oxide check <dosya.ox>`: Derleme öncesi statik tip ve semantik denetimi yapar (hata yakalama).
- [x] `oxide build <dosya.ox> [-o cikti]`: Kodu C'ye çevirip `gcc -O2` ile native binary derler.
- [x] `oxide ast <dosya.ox>`: Programın AST dökümünü alma.
- [x] `oxide repl`: Etkileşimli konsol ortamı.
- [x] Kapsamlı örnekler (`hello.ox`, `fibonacci.ox`, `arrays.ox`, `structs.ox`, `enums_and_match.ox`, mutability testleri, type error testleri).

---

## 🎯 Planlanan Özellikler (Yol Haritası)

### Aşama 1: Veri Yapıları (Tamamlandı)
- [x] Diziler (`[1, 2, 3]`, `arr[i]`, `len(arr)`, `push`, `pop`).
- [x] Rust benzeri `struct` yapıları (`struct Point { x: int, y: int }`).
- [x] `impl` blokları ve metot çağırma (`p.info()`, `self` bağlamı).

### Aşama 2: Tip Sistemi & Pattern Matching (Tamamlandı)
- [x] `enum` tipleri (`enum Status { Pending, Active, Done }`).
- [x] `match` ifadesi ve çoklu desen eşleştirme (Pattern matching).
- [x] **Statik Tip Denetleyici (`src/typecheck.rs`):**
  - Tip uyumsuzlukları (`let x: int = "str"`).
  - Fonksiyon ve metot argüman tipleri/sayıları.
  - Derleme zamanında değişmezlik (`immutable`) ihlalleri.
  - Tanımsız değişken/fonksiyon/struct/enum tespiti.
  - `if/else` ve `match` dal tipi tutarlılığı.

### Aşama 3: Performans ve Bellek Güvenliği (Tamamlandı)
- [x] C Transpiler (`oxide build <dosya.ox>` -> C üretimi -> GCC ile optimize native ELF binary).
- [x] **Bellek Sızıntısı Koruması (Zero Leaks):** Thread-local Ring Buffer Memory Arena mimarisi ile sıfır `malloc`/`free` maliyeti ve sıfır sızıntı.
- [x] CLI Geliştirmeleri: `--emit-c`, `--release`, `-O0..3`, `--cc <compiler>`, `tokens` komutu.

---

## 🔮 Gelecek Yol Haritası ve Eksikler (Upcoming Features)

### Aşama 4: Gelişmiş Döngüler ve Modül Sistemi (Tamamlandı)
- [x] **Gelişmiş Döngü Mekanizmaları:**
  - `for .. in` aralık döngüsü (`for i in 0..10 { ... }`).
  - Koleksiyon ve diziler üzerinde doğrudan gezinme (`for eleman in liste { ... }`).
  - `break` ve `continue` döngü kontrol deyimleri.
  - Derleme zamanı doğrulama (döngü dışında `break`/`continue` kullanımını derlemede engelleme).
  - C Transpiler tarafında tam yerel `for`, `break`, `continue` desteği ve dynamic array entegrasyonu.
- [x] **Modül & Import Sistemi (`src/loader.rs`):**
  - Projeleri çoklu dosyalara bölebilme (`import "matematik.ox";` veya `import "matematik";`).
  - Göreli dosya yolu çözümleme (relative path resolution).
  - Döngüsel içe aktarma tespiti (`circular import detection`, ör: A -> B -> A).
  - Elmas bağımlılık koruması (diamond import protection - aynı modülün birden fazla kez yüklenmesini engelleme).
  - İçe aktarılan fonksiyon, struct, enum ve impl bloklarının tip denetleyici, yorumlayıcı ve native GCC derleyicisi ile kusursuz birleştirilmesi.

### Aşama 5: C FFI ve Standart Kütüphane (I/O) (Tamamlandı)
- [x] **C Kütüphane Entegrasyonu (C FFI - Foreign Function Interface):**
  - `extern "C" { fn sqrt(x: float) -> float; }` blokları ile harici C fonksiyonlarını tanımlama.
  - Derleme zamanında parametre ve dönüş tiplerinin katı denetimi.
  - Native GCC derlemesinde otomatik `-lm` linking ve LibC fonksiyonlarının çakışmasız eşlenmesi (`llabs` vb.).
  - Yorumlayıcıda dahili matematik desteği (`sqrt`, `pow`, `abs`, `sin`, `cos`, `tan`, `floor`, `ceil`, `round`).
- [x] **Giriş / Çıkış (I/O) ve Dosya Yönetimi:**
  - `read_file(yol: str) -> str`: Tüm dosya içeriğini metin olarak okuma.
  - `write_file(yol: str, icerik: str) -> bool`: Dosyaya metin yazma.
  - `read_line() -> str`: Terminalden kullanıcı girdisi alma.
  - Hem yorumlayıcıda hem de GCC native binary'de ring-buffer arena üzerinden sıfır sızıntılı I/O desteği.

---

## 🔮 Mimari Yol Haritası (Endüstri Standardı Derleyici Mimarisi)

### Aşama 6: Derleyici Ara Temsili (HIR / CFG) ve Modül Önbelleği (Tamamlandı)
- [x] **Ek İlkel Tipler (`char` ve `u8` / `byte`):**
  - Karakter literali: `'a'`, `'\n'`, `'\t'`, `'\\'`, `'\''`, `'\0'`.
  - Bayt literali: `b'A'`, `b'\n'`.
  - Tip dönüşümleri: `to_int(c)`, `to_char(n)`, `to_u8(n)` / `to_byte(n)`.
  - Sıralama ve eşitlik karşılaştırmaları (`==`, `!=`, `<`, `<=`, `>`, `>=`).
  - `match` deseni eşleme (`'A' => ...`, `b'A' => ...`).
  - Hem yorumlayıcıda hem de native GCC binary üretiminde tam uyumlu kod üretimi.
- [x] **Modül ve Semantik Önbellek (`ModuleCache`):**
  - Modüllerin çözümlenmiş AST ağaçlarını bellekte saklayan `ModuleCache` yapısı (`src/loader.rs`).
  - Elmas bağımlılıklar ve çoklu dosya derlemelerinde gereksiz disk I/O ve tekrar parse maliyetini sıfırlama.
- [x] **HIR (High-Level Intermediate Representation) & Kontrol Akış Grafiği (CFG):**
  - AST ağacını doğrusal temel bloklara (`BasicBlock`, `HirStatement`) ve sonlandırıcılara (`Terminator::Branch`, `Terminator::Jump`, `Terminator::Return`) dönüştüren ara temsil (`src/hir.rs`).
  - `while`, `for .. in`, `for .. range`, `if .. else` ve `match` ifadelerinin CFG temel bloklarına dönüştürülmesi (desugaring).
  - Geliştirici aracı: `oxide hir <dosya.ox>` ile fonksiyonların CFG ve temel blok akış dökümünü terminale alma.
  - Aşama 7 Non-Lexical Borrow Checker (NLL) ve yaşam süresi analizi için gerekli veri akışı altyapısı hazırlandı.

### Aşama 7: HIR Tabanlı Güçlü Borrow Checker & FFI Güvenlik Sözleşmesi (Tamamlandı)
- [x] **HIR Tabanlı Non-Lexical Borrow Checker (NLL) (`src/borrowck.rs`):**
  - Paylaşılan referans (`&T`) ve değiştirilebilir referans (`&mut T`) ayrımı.
  - Rust Borrow Kuralları: Aynı anda istenildiği kadar `&T` VEYA yalnızca tek bir `&mut T`.
  - Ödünç alınan değişkenin referans varken doğrudan değiştirilmesini (`mutate`) derleme zamanında engelleme.
  - Değişmez (`let`) değişkenlerin `&mut` olarak ödünç alınmasını derleme zamanında engelleme.
  - `&mut` ile ödünç alınan değişkenin doğrudan okunmasını engelleme.
  - **Non-Lexical Lifetimes (NLL):** Referansların son kullanım anı (last use point) CFG üzerinde veri akışıyla tespit edilerek, referansın işi bittiği anda loan otomatik olarak sonlandırılır.
- [x] **FFI Güvenlik Sözleşmesi (Unsafe Boundary & Raw Pointers):**
  - Harici C fonksiyon çağrıları (`extern "C"`) için zorunlu `unsafe { ... }` blok sözleşmesi.
  - Ham gösterici tipleri: `*const T` ve `*mut T`.
  - Ham gösterici dereferansının yalnızca `unsafe` blok içinde yapılması kuralı.
  - `unsafe` blok dışında yapılan C çağrısı veya ham pointer erişiminde net güvenlik hatası (`Güvenlik Hatası [satır:sütun]: ...`).

### Aşama 8: Standart Kütüphane Genişlemesi ve Sistem Araçları (Tamamlandı)
- [x] **Gelişmiş Metin ve Koleksiyon Metotları:**
  - Metin metotları: `s.trim()`, `s.to_upper()`, `s.to_lower()`, `s.contains(sub)`, `s.replace(from, to)`, `s.split(sep)`, `s.char_at(idx)`, `s.len()`.
  - Dizi metotları: `arr.push(elem)`, `arr.pop()`, `arr.len()`.
  - Anahtar-değer sözlük yapısı (`Map`): `map_new()`, `m.set(key, val)`, `m.get(key)`, `m.has(key)`, `m.len()`, `m.keys()`.
  - C codegen tarafında arena tabanlı hash bucket mimarisiyle sıfır malloc maliyeti ve sızıntısız native çalışma.
- [x] **Sistem ve Ortam Değişkenleri (System API):**
  - Komut satırı argümanları: `args() -> [str]`.
  - Ortam değişkenleri okuma: `get_env(anahtar: str) -> str`.
  - Program sonlandırma: `exit(kod: int) -> void`.
  - Hem yorumlayıcıda hem de GCC native derleyicisinde %100 birebir davranış uyumu.

---

### Aşama 9: Paket Yapısı, HIR Optimizasyonları ve Dahili Test Motoru (Tamamlandı)
- [x] **HIR & AST Optimizasyon Geçişleri (Compiler Optimizations - `src/opt.rs`):**
  - **Sabit Katlama (Constant Folding):** Tamsayı, ondalıklı sayı, karakter, bayt ve boolean ifadelerin derleme anında hesaplanıp sabite indirgenmesi (`10 + 20 * 2` -> `50`).
  - **Koşullu Sabit Dallanma İndirgeme:** Sabit değerli boolean dallanmaların (`branch true/false`) doğrudan atlamaya (`jump`) dönüştürülmesi.
  - **Ölü Kod Temizleme (Dead Code Elimination):** Ulaşılamaz temel blokların (`unreachable basic blocks`) CFG akış grafiğinden tamamen temizlenmesi.
  - `oxide hir <dosya.ox> --opt` komutu ile optimize edilmiş CFG dökümünü alma imkanı.
- [x] **Dahili Test Motoru (`oxide test`):**
  - Dosyalardaki ve projelerdeki `test_*` veya `test` fonksiyonlarını tarayıp izole ortamda otomatik çalıştıran test aracı.
  - Renkli konsol raporu (`ok`, `BAŞARISIZ`) ve özet sonuç bildirimi (`X geçti; Y başarısız oldu`).
- [x] **Oxide Proje & Paket Yapısı (`oxide.toml` & `src/project.rs`):**
  - Sıfır dış bağımlılıkla çalışan dahili TOML manifest ayrıştırıcısı.
  - `oxide new <proje_adi>`: Otomatik olarak `oxide.toml`, `src/main.ox` ve `tests/test_basic.ox` içeren yeni proje şablonu üretme.
  - Proje dizininde `oxide run` veya `oxide build` yazıldığında `oxide.toml` manifestini otomatik bularak giriş dosyasını derleme/çalıştırma.

---

### Aşama 10: İsimsiz Fonksiyonlar (Closures / Lambdas) ve Gelişmiş Bellek Kontrolü (Tamamlandı)
- [x] **Closures / Anonim Fonksiyonlar:**
  - `|x: int, y: int| -> int { x + y }`, `|x| x * 2` ve `|| 42` sözdizimi.
  - Birinci sınıf vatandaş fonksiyonlar (First-class functions): Değişkenlere atanabilir ve argüman olarak aktarılabilir.
  - Tip denetleyicisinde `fn(T1, T2) -> R` imza denetimi ve çıkarımı.
  - C codegen tarafında GCC statement expressions ile sıfır ek yükle C fonksiyon göstericisine dönüştürme.
  - Dizi metotlarında filtreleme ve haritalama: `arr.map(...)`, `arr.filter(...)`.
- [x] **Bellek Arenası Yönetim Kontrolleri:**
  - `arena_reset()`: Ağır döngüler veya toplu işlemler sonrasında ring buffer arenasını güvenle sıfırlama mekanizması.

---

### Aşama 11: Generic Tipler ve Parametrik Polimorfizm (Tamamlandı)
- [x] **Generic Fonksiyonlar:**
  - Tek ve çok parametreli generic fonksiyonlar (`fn id<T>(x: T) -> T`, `fn cift<A, B>(ilk: A, ikinci: B) -> A`).
  - Otomatik tip çıkarımı (Type Inference) ve çağrı esnasında argüman tiplerine göre çözümleme.
  - Rust tarzı Turbofish sözdizimi ile açık tip belirtimi (`fn::<T>(...)`).
- [x] **Generic Struct & Impl Yapıları:**
  - Generic struct tanımları (`struct Kutu<T> { deger: T }`, `struct Cift<T, U> { birinci: T, ikinci: U }`).
  - Generic `impl<T> Kutu<T>` blokları ve metot çağrıları (`k.oku()`).
- [x] **Monomorphization Derleme Mimarisi (`src/monomorph.rs`):**
  - C codegen ve yorumlayıcı için derleme zamanında somut tipler üzerinden sıfır çalışma zamanı maliyetli kod çoğaltma (`Kutu__int`, `Kutu__str`, `kimlik__int`, `kimlik__str`).
  - GCC `-O2` derlemesinde doğrudan C struct ve fonksiyonlarına dönüşerek sıfır bellek ve performans kaybı sağlama.

---

### Aşama 12: Hata Yönetimi (`Result<T, E>`, `Option<T>` ve `?` Operatörü) (Tamamlandı)
- [x] **Option Tipleri:**
  - `Option<T>` birinci sınıf statik tip sistemi entegrasyonu.
  - `Some(deger)` ve `None` (ayrıca `Option::Some`, `Option::None`) başlatıcıları.
  - Dahili yardımcı metotlar: `.is_some()`, `.is_none()`, `.unwrap()`, `.unwrap_or(varsayilan)`.
- [x] **Result Tipleri:**
  - `Result<T, E>` statik tip denetimi ve çıkarımı.
  - `Ok(sonuc)` ve `Err(hata)` (ayrıca `Result::Ok`, `Result::Err`) başlatıcıları.
  - Dahili yardımcı metotlar: `.is_ok()`, `.is_err()`, `.unwrap()`, `.unwrap_or(varsayilan)`.
- [x] **Desen Eşleme (`match`):**
  - `match` içinde `Some(v)`, `None`, `Ok(v)` ve `Err(e)` desenlerini değişken bağlama ile yakalama.
- [x] **Hata Yayma Operatörü (`?` - Try Operator):**
  - Fonksiyon içi `let v = islem()?;` şeklinde erken hata/boşluk dönüşü mekanizması.
  - Hem `Option<T>` hem de `Result<T, E>` dönen fonksiyonlarda uyumlu erken çıkış (`early return`).
  - C codegen tarafında GNU C Statement Expressions ile sıfır ek yükle native derleme desteği.
- [x] **Çift Ortam Doğrulaması:** Yorumlayıcı (`oxide run`) ve Native GCC Derleyicisi (`oxide build`) arasında %100 birebir davranış ve çıktı uyumu.

---

### Aşama 13: Trait / Arayüz Mimarisi ve Statik Gönderim (Tamamlandı)
- [x] **Trait Bildirimleri (`trait Name { ... }`):**
  - Zorunlu soyut metot bildirimleri (`fn goster(self) -> str;`).
  - Varsayılan gövdeye sahip metotlar (`default methods`) ve metot mirası.
- [x] **Trait Uygulamaları (`impl Trait for Struct { ... }`):**
  - Struct'lar üzerinde trait arayüzü gerçekleştirimi.
  - Eksik zorunlu metot denetimi ve metot imzası (parametre/dönüş tipi) statik doğrulaması.
  - Varsayılan trait metotlarının otomatik devralınması ve isteğe bağlı ezilmesi (override).
- [x] **Trait Sınırları (Trait Bounds - `<T: Trait>`):**
  - Generic fonksiyonlarda ve struct'larda parametrik trait kısıtlamaları (`fn f<T: Goster>(x: T)`).
  - Derleme zamanında parametre tiplerinin trait sınırını sağlayıp sağlamadığının monomorphizer ve tip denetleyicisinde doğrulanması.
- [x] **Sıfır Maliyetli Statik Gönderim (Zero-Cost Static Dispatch):**
  - Monomorphization aşamasında trait kullanan çağrıların doğrudan somut struct fonksiyonlarına (`Nokta_goster`, `Dikdortgen_goster`) dönüştürülmesi.
  - Sanal metot tablosu (vtable) veya dinamik pointer maliyeti olmadan GCC `-O2` seviyesinde en yüksek hızda native derleme.

---

### Aşama 14: Çoklu Dönüş Değerleri ve Demetler (Tuples: `(A, B, C)`) (Tamamlandı)
- [x] **Tuple Tipleri:** `(int, str, bool)` demet tip tanımları ve literalleri `(42, "oxide", true)`.
- [x] **Demet Eleman Erişimi ve Değiştirilebilirlik:** İndeks tabanlı nokta erişimi (`t.0`, `t.1`) ve değiştirilebilir demet elemanı güncelleme (`mt.0 = 555;`).
- [x] **Demet Ayrıştırma (Destructuring):** `let (x, y, unvan) = koordinat();` ve içsel mutability (`let (mut sayac, carpici) = ...;`).
- [x] **Fonksiyonlardan Çoklu Değer Dönüşü:** `fn bolme_ve_kalan(a: int, b: int) -> (int, int)`.
- [x] **Demet Desen Eşleme (Tuple Pattern Matching):** `match` ifadesinde demetleri doğrudan eşleme (`(0, 0) => ...`, `(1, y) => ...`, `_ => ...`).
- [x] **Native C Transpiler & GCC Entegrasyonu:** Her özgün demet tipi için otomatik C `struct` üretimi (`_OxTuple_...`), compound literal başlatma ve C11 `_Generic` ile doğrudan `println(t)` desteği.

---

### Aşama 15: String Formatlama & Metin İnterpolasyonu (String Interpolation & Advanced Formatting) (Tamamlandı)
- [x] **String İnterpolasyonu:** `f"Merhaba {isim}, puanın: {skor}"` ve süslü parantez içinde tam ifade desteği (`f"{x} + {y} = {x + y}"`, `f"Harf: {s.to_upper()}"`).
- [x] **Çoklu Tip İnterpolasyon Desteği:** Tamsayı, ondalıklı sayı, karakter, boolean, Option (`Some`/`None`), Result (`Ok`/`Err`), Tuple (`(a, b, c)`) tiplerinin metin formatına otomatik dönüştürülmesi.
- [x] **Kaçış Karakterleri (Escaped Braces):** `{{` ve `}}` kullanarak metin içinde süslü parantez bırakabilme (`f"JSON: {{\"id\": {id}}}"`).
- [x] **Formatlama Fonksiyonu:** Rust tarzı `format("Şablon: {}, {}", arg1, arg2)` fonksiyonu ve dinamik metin üretimi.
- [x] **Bellek Arenası Entegrasyonu & C Transpiler:** Thread-local ring buffer arenası (`_ox_arena_alloc`) ile sıfır bellek sızıntısı ve GCC `-O2` derlemesinde yüksek performanslı native çalışma.
- [x] **Çift Ortam Doğrulaması:** Yorumlayıcı (`oxide run`) ve Native GCC Derleyicisi (`oxide build`) arasında %100 birebir davranış ve çıktı uyumu.

---

### Aşama 16: Dilimler ve Aralık İndeksleme (Slices & Range Indexing: `&[T]`, `&str`, `arr[bas..son]`, `s[bas..son]`) (Tamamlandı)
- [x] **Aralık İndeksleme Sözdizimi:** `arr[bas..son]`, `arr[bas..]`, `arr[..son]`, `arr[..]` ve metinler için `s[bas..son]`.
- [x] **Sıfır Kopyalama Dilim Mimarisi (Zero-Copy Slices):** Veriyi kopyalamadan işaretçi ve uzunluk (`ptr + len`) taşıyan güvenli dilim nesnesi (`Value::Slice`, `Value::StrSlice`).
- [x] **Sınır Denetimi (Bounds Checking):** Dilim aralığının dizi/metin sınırlarını aşmadığını kontrol eden güvenlik denetimleri ve çalışma zamanı koruması.
- [x] **Tip Denetleyicisi & Monomorphizer Entegrasyonu:** Dilim tipleri (`&[T]`, `&str`), otomatik dizi-dilim uyumluluğu (coercion) ve fonksiyon parametresi olarak dilim kabulü (`fn topla(sayilar: &[int]) -> int`).
- [x] **Dahili Metotlar & Döngüler:** `.len()`, `.to_array()`, `.to_vec()`, `.to_str()`, `.trim()`, `.to_upper()`, `for x in dilim { ... }` döngü desteği.
- [x] **Native C Codegen Desteği:** GCC derleyicisi için optimize `_OxSlice` ve `_OxStrSlice` struct temsilleri, `_ox_slice_create`, `_ox_str_slice_create`, C11 `_Generic` uyumu ve sıfır bellek sızıntılı çalışma.
- [x] **Çift Ortam Doğrulaması:** Yorumlayıcı (`oxide run`) ve Native GCC Derleyicisi (`oxide build`) arasında %100 birebir davranış ve çıktı uyumu (27/27 entegrasyon testi başarılı).

---

### Aşama 17: Operatör Aşırı Yükleme & Özel Traitler (Operator Overloading: `Add`, `Sub`, `Mul`, `Div`, `Neg`, `Index`, `Eq`) (Tamamlandı)
- [x] **Yerleşik Operatör Traitleri:** Standart operatör traitlerinin dil düzeyinde tanınması (`Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`, `Not`, `Index`, `Eq`).
- [x] **Kullanıcı Tanımlı Struct'lar İçin Operatör Ezme:** `impl Add for Vector2D { fn add(self, other: Vector2D) -> Vector2D { ... } }` şeklinde `+`, `-`, `*`, `/`, `[]`, `==`, `!=`, `-` (tek terimli) operatörlerinin struct tipleriyle tam uyumlu çalışması.
- [x] **İndeks Ataması (`IndexSet`):** `v[0] = 555;` şeklinde `index_set(mut self, idx: int, val: int)` metodu üzerinden değiştirilebilir indeks atama desteği.
- [x] **Statik Tip Çözümleme & Monomorphizer:** İkili (binary), tekli (unary) ve indeksleme ifadelerinde operatör metodunun ve dönüş tipinin derleme zamanında çözümlenmesi.
- [x] **İfade Zincirleme & Statement Expressions:** `(v1 + v2) * 2` gibi zincirleme karmaşık ifadelerin GNU C Statement Expressions (`({ ... })`) ile optimize derlenmesi.
- [x] **Trait Bound ile Generic Kullanım:** `<T: Add>` trait sınırına sahip fonksiyonlarda operatörlerin ve metotların sıfır maliyetli statik gönderimi (`topla_generic(v1, v2)`).
- [x] **Çift Ortam Doğrulaması:** Yorumlayıcı (`oxide run`) ve Native GCC Derleyicisi (`oxide build`) arasında %100 birebir davranış ve çıktı uyumu (28/28 entegrasyon testi başarılı).

---

### Aşama 18: Paket Yönetimi, `target/` Mimarisi, `oxide.lock` & `clean` (Tamamlandı)
- [x] **Rust/Cargo Tarzı `target/` Mimarisi:**
  - Derleme çıktılarının kaynak dizini kirletmemesi için `target/debug/` ve `target/release/` dizin yapısı.
  - Transpile edilen ara C dosyalarının `target/<profil>/c_gen/<proje>.c` içine gizlenmesi.
  - Nihai çalıştırılabilir binary dosyalarının `target/<profil>/<proje>` konumuna yerleştirilmesi.
- [x] **`oxide.lock` Kilit Dosyası:**
  - Paket adı, sürüm, giriş dosyası ve kaynak bilgisini tutan otomatik oluşturulan ve güncellenen deterministik kilit dosyası.
- [x] **Git Koruması (`.gitignore`):**
  - `oxide new` ile oluşturulan projelerde otomatik `.gitignore` (`/target/`).
  - `target/` klasörü altına otomatik `target/.gitignore` (`*`) yerleştirilerek git kirliliğinin engellenmesi.
- [x] **`oxide clean` Komutu:**
  - Proje veya mevcut çalışma dizinindeki `target/` klasörünü ve tüm derleme kalıntılarını güvenle temizleyen komut.
- [x] **Otomatik Entegrasyon Testi:** `test_project_target_lock_and_clean` ile 29/29 test başarıyla geçti.

---

### Aşama 19: Akıllı İşaretçiler & Yığın Yönetimi (Smart Pointers: `Box<T>`, `Rc<T>`, `RefCell<T>`) (Tamamlandı)
- [x] **`Box<T>` Yığın Tahsisi:** Verileri heap/arena üzerinde tutan tekil sahiplikli akıllı işaretçi tipi, `Box::new(val)`, `*b` dereferans okuma ve `*b = yeni_deger` dereferans atama (`DerefAssign`) desteği.
- [x] **`Rc<T>` Paylaşımlı Referans Sayımı:** Çoklu sahipliğe izin veren referans sayımlı işaretçi (`Rc::new(val)`), `rc.clone()` ile sayaç artırma, `Rc::strong_count(rc)` ve `*rc` dereferans okuma.
- [x] **`RefCell<T>` İçsel Değiştirilebilirlik (Interior Mutability):** Değişmez referanslar üzerinden `*cell.borrow()` ile okuma ve `*cell.borrow_mut() = val` ile içsel mutasyon desteği.
- [x] **Özyinelemeli (Recursive) Veri Yapıları:** Bağlı liste (`LinkedList`), ikili ağaç (`BinaryTree`) gibi struct yapılarının `Box<T>` ve `Option<Box<T>>` ile güvenli tanımlanabilmesi.
- [x] **Otomatik Dereferans (Auto-deref):** Nokta operatörü ile struct alanlarına ve metotlarına erişirken `ptr->field` ve `ptr.method()` şeklinde otomatik işaretçi çözümü.
- [x] **Native C Transpiler & İleri Bildirimler (Forward Declarations):** Döngüsel ve özyinelemeli struct tanımları için C seviyesinde `typedef struct Node Node;` ileri bildirimleri, optimize `_OxRc` yapısı ve GCC `-O2` entegrasyonu.
- [x] **Otomatik Entegrasyon Testi:** `test_smart_pointers_and_recursive_structs` ile 30/30 test başarıyla geçti.

---

### Aşama 20: Derleme Zamanı Sabitleri & Standart Dosya/Dizin Sistemi (`const` Evaluator, `std::fs`, `std::env`) (Tamamlandı)
- [x] **`const` İfadeleri & Derleme Zamanı Hesaplama:** `const BUFFER_SIZE: int = 1024 * 4;` ve `const GREETING: str = APP_NAME + " Sistem Araci";` şeklinde derleme zamanında tip denetleyicisi (`eval_const_expr`) tarafından çözülen ve inlined edilen sabitler.
- [x] **Blok İçi Yerel `const` Desteği:** Fonksiyon veya blok gövdesinde `const CARPAN: int = 10 + 2;` şeklinde yerel sabit tanımlayabilme.
- [x] **Standart Dosya Sistemi Modülü (`fs::`):**
  - `fs::write(yol, icerik) -> Result<void, str>` (dosya oluşturma/yazma).
  - `fs::read_to_string(yol) -> Result<str, str>` (dosyadan metin okuma).
  - `fs::exists(yol) -> bool` (dosya/dizin varlık kontrolü).
  - `fs::remove_file(yol) -> Result<void, str>` (dosya silme).
  - `fs::create_dir(yol) -> Result<void, str>` (klasör oluşturma).
- [x] **Sistem & Çevre Değişkenleri (`env::`):**
  - `env::args() -> [str]` (komut satırı argümanları).
  - `env::var(anahtar) -> Option<str>` (çevre değişkeni okuma).
  - `env::current_dir() -> Result<str, str>` (çalışma dizini sorgulama).
- [x] **Native C Transpiler & GCC Entegrasyonu:** POSIX/C standart kütüphane dosya ve ortam fonksiyonları (`fopen`, `fread`, `fwrite`, `fclose`, `remove`, `access`, `mkdir`, `getenv`, `getcwd`) ile yorumlayıcı arasında %100 birebir davranış uyumu.
- [x] **Otomatik Entegrasyon Testi:** `test_const_and_filesystem` ile 31/31 test başarıyla geçti.

---

### Aşama 21: Eşzamanlılık & İş Parçacıkları (Concurrency: `thread::spawn`, `channel`, `Mutex<T>`) (Tamamlandı)
- [x] **`thread::spawn` ile İş Parçacığı Başlatma:** Fonksiyon veya closure alarak arka planda bağımsız çalışan thread desteği (`thread::spawn(|| { ... })`), `thread::sleep(ms)` uyku süresi ve `.join().unwrap()` ile thread sonucunu alma.
- [x] **Mesajlaşma Kanalları (`channel`):** Thread'ler arası güvenli veri iletimi sağlayan Sender/Receiver kuyruğu (`let (tx, rx) = channel();`), `tx.send(val)` ile mesaj iletme ve `rx.recv().unwrap()` ile bloklayıcı mesaj alma.
- [x] **`Mutex<T>` ile Güvenli Paylaşımlı Bellek:** Thread'ler arasında güvenli veri paylaşımı, `Mutex::new(val)` başlatıcısı, `m.lock()` ile kilit koruması ve recursive mutex desteği.
- [x] **Thread-Safe Evaluator Mimarisi:** `Value` enum'ı `Arc<Mutex<...>>` (Shared) modeline taşınarak %100 `Send + Sync` yapıldı.
- [x] **Native C Transpiler & POSIX pthreads Entegrasyonu:** POSIX Threads (`pthread_create`, `pthread_join`, `pthread_mutex`, `pthread_cond`) ile C transpiler uyumu, GCC otomatik `-pthread` bayrağı ile native derleme.
- [x] **Otomatik Entegrasyon Testi:** `test_concurrency_and_threads` ile 32/32 test başarıyla geçti.

---

### Aşama 22: Ağ & Soket Programlama (Network & Sockets: `TcpListener`, `TcpStream`) (Tamamlandı)
- [x] **TCP Sunucu Soketi (`net::TcpListener`):**
  - Belirtilen IP ve port üzerinde dinleme başlatma (`TcpListener::bind("127.0.0.1:8080") -> Result<TcpListener, str>`).
  - Gelen istemci bağlantılarını kabul etme (`listener.accept() -> Result<TcpStream, str>`).
- [x] **TCP İstemci Soketi (`net::TcpStream`):**
  - Uzak sunucuya TCP bağlantısı kurma (`TcpStream::connect("127.0.0.1:8080") -> Result<TcpStream, str>`).
  - Veri gönderme (`stream.write(veri: str) -> Result<int, str>`).
  - Veri okuma (`stream.read() -> Result<str, str>`).
  - Bağlantıyı güvenli sonlandırma (`stream.close() -> void`).
- [x] **Çoklu İş Parçacıklı Ağ Mimarisi:** `thread::spawn` ile sunucuyu arka planda çalıştırarak istemcilerle eşzamanlı haberleşme.
- [x] **Native Berkeley Sockets Transpiler & GCC Entegrasyonu:**
  - POSIX `<sys/socket.h>`, `<netinet/in.h>`, `<arpa/inet.h>` C runtime entegrasyonu.
  - C seviyesinde `_ox_tcp_listener_bind`, `_ox_tcp_listener_accept`, `_ox_tcp_stream_connect`, `_ox_tcp_stream_write`, `_ox_tcp_stream_read`, `_ox_tcp_stream_close` fonksiyonları.
  - GCC `-O2` seviyesinde optimize native binary üretimi.
- [x] **Otomatik Entegrasyon Testi:** `test_network_and_tcp_sockets` ile 33/33 test başarıyla geçti.

---

### Aşama 23: HTTP Sunucusu / REST API & JSON Serileştirme (Sıradaki Hedef)
- [ ] **HTTP Protokol Çözümleyici & İstek/Yanıt:** `http::Request` (metot, yol, başlıklar, gövde) ve `http::Response` (durum kodu, başlıklar, gövde).
- [ ] **Yerleşik JSON Parser / Stringifier:** Nesneleri, dizileri ve struct'ları JSON metnine (`json::stringify`) ve JSON metnini veri tiplerine (`json::parse`) dönüştürme.
- [ ] **Hafif REST API Web Sunucusu:** `http::Server` ile yönlendirme (routing), GET/POST endpoint'leri ve statik dosya sunumu.



