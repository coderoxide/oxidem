# Oxide Derleyici Mimarisi (Compiler Architecture)

Oxide derleyicisi, modüler ve genişletilebilir bir boru hattı (compiler pipeline) şeklinde tasarlanmıştır.

```
Kaynak Kod (.ox)
       │
       ▼
 ┌───────────┐
 │   Lexer   │  ──> Token Dizisi (Tokens + Spans)
 └───────────┘
       │
       ▼
 ┌───────────┐
 │  Parser   │  ──> Pratt Parser Tabanlı AST (Soyut Sözdizimi Ağacı)
 └───────────┘
       │
       ▼
 ┌──────────────┐
 │ ModuleLoader │  ──> Çoklu dosya çözümü, döngüsel import engelleme
 └──────────────┘
       │
       ▼
 ┌─────────────┐
 │ TypeChecker │  ──> Statik Tip, Fonksiyon İmzaları & Değişmezlik Denetimi
 └─────────────┘
       │
       ├──────────────────────┬──────────────────────┐
       ▼                      ▼                      ▼
 ┌───────────┐          ┌───────────┐          ┌───────────┐
 │ Evaluator │          │    HIR    │          │ Codegen C │
 │(Yorumlama)│          │(CFG / BBs)│          │ (Derleme) │
 └───────────┘          └───────────┘          └───────────┘
       │                      │                      │
       ▼                      ▼                      ▼
 Anında Yürütme         Dataflow & CFG         C11 Kaynak Kodu
 (`oxide run`)          Borrowck Temeli              │
                        (`oxide hir`)                ▼
                                               ┌─────────────┐
                                               │ GCC Backend │ (-O2 -lm)
                                               └─────────────┘
                                                     │
                                                     ▼
                                               Native Binary
                                              (`oxide build`)
```

---

## 1. Modül Detayları

### 1.1. `src/span.rs` (Konum Takibi)
* Her sözcüğün ve sözdizim ağacı düğümünün kaynak dosyadaki satır (`line`) ve sütun (`col`) koordinatlarını tutar.
* Tüm hata mesajlarında kullanıcıya kesin hata yerini gösterir (`Hata [satır:sütun]: ...`).

### 1.2. `src/token.rs` (Sözcük Tanımları)
* `TokenKind`: Anahtar sözcükler (`Fn`, `Let`, `Mut`, `Struct`, `Impl`, `Enum`, `Match`), operatörler (`+`, `-`, `*`, `/`, `==`, `::`, `=>`), literaller (`IntLit`, `FloatLit`, `StringLit`, `BoolLit`) ve ayraçlar (`(`, `)`, `{`, `}`, `[`, `]`).

### 1.3. `src/lexer.rs` (Sözcüksel Analizci)
* UTF-8 karakter akışını tek geçişte okur.
* Sayı tiplerini (`i64` veya `f64`), kaçış karakterli metin dizgilerini (`\n`, `\t`, `\"`), tek satırlı yorumları (`//`) ve çift karakterli sembolleri (`->`, `=>`, `::`, `==`, `!=`, `<=`, `>=`) çözümler.

### 1.4. `src/ast.rs` (Soyut Sözdizimi Ağacı)
* İfadeleri (`Expr`), deyimleri (`Stmt`), blokları (`Block`), fonksiyonları (`Function`), yapıları (`StructDef`), numaralandırmaları (`EnumDef`) ve desenleri (`Pattern`) temsil eden cebirsel veri yapıları (ADTs).

### 1.5. `src/parser.rs` (Pratt Parser & Recursive Descent)
* Karmaşık ikili ve tekli ifadeleri Vaughan Pratt'ın öncelik tırmanışlı (precedence climbing) algoritmasıyla çözümler.
* Rust tarzı blok dönüşlerini (`trailing expression`), struct başlatıcılarını (`Point { x: 10 }`) ve metot zincirlerini (`p.scale().area()`) çözümler.

### 1.6. `src/loader.rs` (Modül Yükleyici)
* Dosya bağımlılıklarını (`import "dosya.ox";`) göreli yollarla (relative paths) çözer.
* Döngüsel içe aktarmaları (`import_stack`) tespit ederek sonsuz döngüleri engeller.
* Elmas bağımlılıkları (`visited`) tekilleştirerek aynı modülün birden fazla kez yüklenmesini önler.

### 1.7. `src/typecheck.rs` (Statik Tip ve Semantik Denetleyici)
* Program çalıştırılmadan veya derlenmeden önce 2 geçişli (two-pass) analiz yapar.
* Değişken, fonksiyon ve metot tiplerini, parametre uyumluluğunu, değişmezlik ihlallerini (`let` vs `let mut`), döngü kapsamlarını (`break`/`continue` derinliği) ve C FFI imzalarını doğrular.

### 1.8. `src/eval.rs` (Çalışma Zamanı ve Yorumlayıcı)
* **Kapsam Ağacı (`Environment`):** İç içe geçmiş değişken kapsamlarını (Lexical Scopes) yönetir.
* **Değişmezlik Denetimi:** `let` ile tanımlanmış bir değişkene, dizi indeksine veya struct alanına atama yapıldığında çalışma zamanında müdahale eder.
* **Referans Yönetimi:** Diziler ve struct'lar için içsel `Rc<RefCell<...>>` kullanarak paylaşımlı ve güvenli bellek modeli sağlar.

### 1.9. `src/codegen_c.rs` (C Transpiler & Native Code Generator)
* AST ağacını standart C11 koduna dönüştürür.
* İfade bloklarını GNUC statement expression `({ ... })` yapılarına çevirir.
* Çoklu tip desteğini C11 `_Generic` makrolarıyla tür güvenli (type-safe) biçimde sağlar.
* Ring Buffer Memory Arena mimarisi ile `malloc`/`free` maliyeti olmadan sıfır sızıntılı bellek yönetimi sunar.

### 1.10. `src/hir.rs` (High-Level Intermediate Representation & CFG)
* AST ağacını doğrusal temel bloklara (`BasicBlock`), ifadelere (`HirStatement`) ve sonlandırıcılara (`Terminator::Branch`, `Jump`, `Return`) dönüştürür.
* `while`, `for .. in`, `for .. range`, `if/else` ve `match` ifadelerini açık CFG akışına desugar eder.
* `oxide hir <dosya.ox>` komutu ile fonksiyonların CFG yapısını dökebilir.
* Non-Lexical Borrow Checker (NLL) ve liveness veri akışı analizinin omurgasını oluşturur.

### 1.11. `src/borrowck.rs` (Dataflow Borrow Checker & Non-Lexical Lifetimes)
* HIR kontrol akış grafiği üzerinden veri akışı (dataflow) analizi yapar.
* Değişkenlerin son kullanım noktasını (`compute_last_uses`) hesaplayarak NLL (Non-Lexical Lifetimes) sağlar.
* `&T` ve `&mut T` kurallarını, değişmez değişken ödünçlerini ve aliasing zincirlerini derleme anında doğrular.
* `extern "C"` ve ham pointer erişimleri için `unsafe { ... }` sınırını denetler.

---

## 2. Sıfır Dış Bağımlılık (Zero External Dependencies)

Oxide'ın tüm derleyici ve yorumlayıcı kodu **harici hiçbir crate olmadan**, yalnızca Rust standart kütüphanesi (`std`) kullanılarak yazılmıştır. Bu sayede:
1. Derleme hızı anlıktır (<0.5 saniye).
2. Projede tedarik zinciri (supply chain) güvenlik açığı riski yoktur.
3. Çalıştırılabilir derleyici ikilisi son derece hafiftir.

---

## 3. Gelecek Mimari Evrim: HIR & Dataflow Borrow Checking

### 3.1. Neden AST Yerine HIR (High-Level Intermediate Representation)?
Soyut Sözdizimi Ağacı (AST) sözdizimsel olarak hiyerarşik bir ağaçtır; fakat bellek güvenliği ve sahiplik analizi zamansal bir **kontrol akışı (Control Flow Graph - CFG)** gerektirir.
* Doğrudan AST üzerinde Borrow Checker koşmak, karmaşık `match`, iç içe döngüler ve erken `return` durumlarında hatalı reddetmelere (false-positive) yol açar (Rust'ın ilk yıllarındaki AST borrowck problemi).
* Bu nedenle Oxide, AST'yi temel bloklara (`BasicBlock`) ve açık dallanma etiketlerine (`jump`, `branch_if`) desugar eden **HIR/CFG** katmanına geçiş yapacaktır.

### 3.2. FFI Güvenlik Modeli
* C tarafından gelen ham bellek göstericileri için `*const T` ve `*mut T` (raw pointer) ayrımı.
* C FFI çağrıları ve ham pointer dereferansları için `unsafe { ... }` güvenlik bariyeri.
