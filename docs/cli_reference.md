# Oxide CLI ve Komut Satırı Referansı (CLI Reference)

Oxide derleyicisi, geliştiricilere hem hızlı prototipleme hem de üretim için optimize edilmiş ikili derleme yetenekleri sunan güçlü bir komut satırı arayüzüne sahiptir.

---

## 1. Komut Özeti

```bash
cargo run -- <alt-komut> [seçenekler] [argümanlar]
```

| Alt Komut | Kullanım | Açıklama |
| :--- | :--- | :--- |
| `run` | `cargo run -- run <dosya.ox>` | Kaynak dosyayı tip denetiminden geçirip yorumlayarak çalıştırır. |
| `check` | `cargo run -- check <dosya.ox>` | Kaynak dosyanın derleme zamanı tip ve semantik denetimini yapar. |
| `build` | `cargo run -- build <dosya.ox> [seçenekler]` | Kodu C'ye çevirir ve derler (`--emit-c` sadece C üretir). |
| `tokens` | `cargo run -- tokens <dosya.ox>` | Yalnızca Lexer çıktısını (token akışını) satır/sütun dökümüyle basar. |
| `ast` | `cargo run -- ast <dosya.ox>` | Programın çözümlenmiş AST dökümünü ekrana basar. |
| `repl` | `cargo run -- repl` | Etkileşimli terminal konsolunu başlatır. |
| `help` | `cargo run -- help` | Yardım mesajını görüntüler. |

---

## 2. Detaylı Komut Kullanımı

### 2.1. `run` (Yorumlayıcı Modu)
Hızlı geliştirme ve test döngüleri için idealdir. Kodu derleme adımına sokmadan önce statik tip denetiminden geçirir ve AST üzerinden doğrudan yürütür:

```bash
cargo run -- run examples/hello.ox
```

### 2.2. `check` (Statik Tip Denetimi)
Kodu çalıştırmadan veya derlemeden önce tüm tip uyumsuzluklarını, tanımsız değişkenleri ve değişmezlik (immutability) ihlallerini anında yakalar:

```bash
cargo run -- check examples/fibonacci.ox
# Çıktı: ✔ [Başarılı] Derleme zamanı tip ve semantik denetimi geçti. (0 hata)
```

### 2.3. `tokens` (Lexer Token Akışı)
Özellikle sözcüksel hataları ve Pratt Parser öncesinde token sınırlarını denetlemek için kullanılır:

```bash
cargo run -- tokens examples/hello.ox
```
**Çıktı Örneği:**
```text
=== Token Akışı: 'examples/hello.ox' (Toplam: 75) ===
   1: [1: 1] Fn
   2: [1: 4] Ident("greet")
   3: [1: 9] LParen
   4: [1:10] Ident("name")
   ...
```

### 2.4. `build` (Native Derleme ve Bayraklar)
Kaynak kodu C11'e transpiled eder ve makine koduna (ELF binary) derler.

#### Kullanışlı Bayraklar (Flags):
* **`-o <dosya>`:** Çıktı ikili dosyasının adını veya yolunu belirler.
* **`--emit-c` / `-C`:** C derleyicisini (GCC/Clang) arka planda çalıştırmadan, yalnızca üretilen `.c` kaynak dosyasını diske bırakır. Transpiler çıktısını incelemek için mükemmeldir.
* **`--release`:** GCC/Clang'e `-O3` maksimum optimizasyon bayrağını iletir.
* **`-O0`, `-O1`, `-O2`, `-O3`:** Özel optimizasyon seviyesi seçimi (varsayılan: `-O2`).
* **`--cc <derleyici>`:** Sistemdeki varsayılan `gcc` yerine `clang` veya cross-compiler seçimi yapmayı sağlar.

```bash
# 1. Yalnızca üretilen C kodunu görmek için:
cargo run -- build examples/hello.ox --emit-c

# 2. Release (-O3) modunda derleme:
cargo run -- build examples/fibonacci.ox --release

# 3. Clang kullanarak derleme:
cargo run -- build examples/structs.ox --cc clang -o my_structs
./my_structs
```

### 2.3. `ast` (Sözdizim Ağacı Dökümü)
Derleyici geliştiricileri veya kodun nasıl çözümlendiğini görmek isteyenler için AST ağacının Rust `Debug` formatında çıktısını verir:

```bash
cargo run -- ast examples/hello.ox
```

### 2.4. `repl` (Etkileşimli Konsol)
Hızlı ifadeleri, aritmetik işlemleri veya blokları anında denemek için kullanılır:

```bash
cargo run -- repl
```
```text
Oxide v0.1.0 REPL
Çıkmak için 'exit' veya 'quit' yazın.

ox> 10 + 25 * 2
=> 60
ox> let mut arr = [1, 2, 3]; push(arr, 99); arr;
=> [1, 2, 3, 99]
ox> exit
```
