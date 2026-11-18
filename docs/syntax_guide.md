# Oxide Sözdizimi Rehberi (Syntax Guide)

Oxide, Rust sözdiziminden ve ifade odaklı (expression-oriented) tasarım felsefesinden ilham almıştır.

---

## 1. Değişken Tanımlama ve Değişmezlik (Immutability)

Oxide'da değişkenler varsayılan olarak **değiştirilemezdir (immutable)**. Bir değişkene sonradan değer atamak istiyorsanız `mut` anahtar sözcüğü kullanılmalıdır.

```rust
// Değiştirilemez (Immutable) Değişken
let x = 10;
// x = 20; // HATA! Değiştirilemez 'x' değişkenine yeniden atama yapılamaz!

// Değiştirilebilir (Mutable) Değişken
let mut y = 20;
y = 35; // Geçerli!

// İsteğe bağlı açık tip belirtimi
let mut sayac: int = 0;
let mesaj: str = "Selam";
let pi: float = 3.14159;
let aktif: bool = true;
```

---

## 2. Temel Veri Tipleri

| Tip | Rust Karşılığı | C Karşılığı | Açıklama |
| :--- | :--- | :--- | :--- |
| `int` | `i64` | `int64_t` | 64-bit işaretli tamsayı |
| `float` | `f64` | `double` | 64-bit IEEE-754 ondalıklı sayı |
| `bool` | `bool` | `bool` | `true` veya `false` mantıksal değer |
| `str` | `String` / `&str` | `const char*` | UTF-8 metin dizgisi |
| `void` | `()` | `void` | Boş/Dönüşsüz tip |
| `[T]` | `Vec<T>` | `T*` | `T` tipinde elemanlar barındıran dinamik dizi |
| `&[T]` | `&[T]` | `_OxSlice` | `T` dizisi üzerinde sıfır kopyalama bellek dilimi (pointer + len) |
| `&str` | `&str` | `_OxStrSlice` | Metin üzerinde sıfır kopyalama alt-metin dilimi (pointer + len) |

---

## 3. İfade Odaklı Blok Yapısı (Expression-Oriented Blocks)

Oxide'da her kod bloğu `{ ... }` bir ifadedir (expression). Blok içerisindeki **son satıra noktalı virgül (`;`) koymazsanız**, blok o değeri döndürür.

```rust
let sonuc = {
    let a = 15;
    let b = 25;
    a + b // Noktalı virgül yok! 'sonuc' 40 değerini alır.
};

println("Sonuç:", sonuc); // Çıktı: 40
```

Eğer son satıra noktalı virgül konulursa (`a + b;`), ifadenin sonucu yoksayılır ve blok `void` (`()`) döner.

---

## 4. Kontrol Akışı (Control Flow)

### 4.1. `if` / `else if` / `else`
`if` bir ifade olduğundan doğrudan değer döndürebilir:

```rust
let durum = true;
let sayi = if durum {
    100
} else {
    0
};

println("Sayı:", sayi); // 100
```

Koşullu dallanma:
```rust
let puan = 85;

if puan >= 90 {
    println("Harika!");
} else if puan >= 70 {
    println("Başarılı.");
} else {
    println("Tekrar dene.");
}
```

### 4.2. `while` Döngüsü
Belirli bir koşul sağlandığı sürece kod bloğunu yineler:

```rust
let mut i = 0;
while i < 5 {
    println("Sayaç:", i);
    i = i + 1;
}
```

### 4.3. `for .. in` Aralık Döngüsü (Range)
`start..end` sözdizimi ile belirli bir sayısal aralıkta güvenli döngü kurulur:

```rust
for i in 0..5 {
    println("Adım:", i); // 0, 1, 2, 3, 4
}
```

### 4.4. `for .. in` Koleksiyon Döngüsü (Iterables)
Diziler üzerinde doğrudan eleman bazlı dolaşım sağlar:

```rust
let sayilar = [10, 20, 30];
for num in sayilar {
    println("Eleman:", num);
}
```

### 4.5. `break` ve `continue`
* **`break;`**: Döngüyü anında sonlandırır ve döngüden çıkar.
* **`continue;`**: Döngünün o anki iterasyonunu atlar ve bir sonraki adıma geçer.
* *Not:* Döngü dışında `break` veya `continue` kullanımı derleme zamanında tip denetleyicisi tarafından yakalanır.

```rust
for i in 0..10 {
    if i % 2 == 0 {
        continue; // Çift sayıları atla
    }
    if i > 7 {
        break; // 7'den büyük ilk tek sayıda çık
    }
    println("Tek:", i);
}
```

---

## 5. Fonksiyonlar

Fonksiyonlar `fn` anahtar sözcüğü ile tanımlanır. Parametreler `isim: tip` biçimindedir. Dönüş tipi `-> tip` ile belirtilir (dönüşsüzse atlanabilir).

```rust
// İki sayıyı toplayıp int dönen fonksiyon
fn topla(a: int, b: int) -> int {
    a + b // Rust tarzı örtük return (implicit return)
}

// Erken dönüş (early return) desteği
fn mutlak_deger(x: int) -> int {
    if x < 0 {
        return -x;
    }
    x
}

// Parametrede 'mut' desteği
fn sayaci_artir(mut val: int) -> int {
    val = val + 1;
    val
}
```

---

## 6. Dahili Fonksiyonlar (Built-in Functions)

* **`println(...)`**: Verilen tüm argümanları aralarında boşluk bırakarak yazdırır ve alt satıra geçer.
* **`print(...)`**: Argümanları alt satıra geçmeden yazdırır.
* **`assert(kosul, [mesaj])`**: Koşul `true` değilse programı hata ile durdurur.
* **`len(koleksiyon)`**: Dizinin veya metnin uzunluğunu (`int`) döndürür.
* **`push(dizi, eleman)`**: Dizi sonuna yeni eleman ekler.
* **`pop(dizi)`**: Dizi sonundaki elemanı çıkarıp geri döner.
* **`read_file(yol)`**: Verilen yoldaki dosyanın tüm içeriğini metin (`str`) olarak döner.
* **`write_file(yol, icerik)`**: Belirtilen dosyaya metin yazar; başarılıysa `true`, değilse `false` döner.
* **`read_line()`**: Terminalden kullanıcıdan tek satır girdi okur ve metin (`str`) olarak döner.

---

## 7. Modül ve Çoklu Dosya Sistemi (`import`)

Oxide, projeleri birden fazla dosyaya bölüp organize etmeyi sağlayan bir modül sistemine sahiptir.

### 7.1. Dosya İçe Aktarma
`import "dosya.ox";` veya uzantısız `import "dosya";` sözdizimi ile başka bir dosyadaki fonksiyonlar, struct'lar ve enum'lar mevcut kapsama dahil edilir:

```rust
// matematik.ox
fn topla(a: int, b: int) -> int {
    a + b
}

// main.ox
import "matematik.ox";

fn main() {
    let sonuc = topla(10, 20);
    println("Toplam:", sonuc);
}
```

### 7.2. Özellikler ve Güvenlik
* **Göreli Yollar (Relative Paths):** Dosyalar, o anki çağıran dosyanın bulunduğu dizine göre aranır.
* **Döngüsel İçe Aktarma Koruması:** `A -> B -> A` gibi döngüsel bağımlılıklar derleme öncesinde tespit edilir ve net bir hata ile raporlanır.
* **Elmas Bağımlılık (Diamond Import) Koruması:** Aynı dosya birden fazla modül tarafından içe aktarılsa bile (ör: `A -> C`, `B -> C`, `Main -> A, B`), `C` yalnızca bir kez yüklenir; sembol çakışması yaşanmaz.
* **C Native Derleme:** `oxide build main.ox` komutu, içe aktarılan tüm bağımlılıkları tek bir optimize native binary dosyasına derler.

---

## 8. C FFI (Foreign Function Interface)

Oxide, C kütüphanelerindeki (`LibC`, `math.h`, harici grafik ve veritabanı kütüphaneleri) fonksiyonları doğrudan bağlamak için Rust tarzı `extern "C"` bloklarını destekler:

```rust
extern "C" {
    fn sqrt(x: float) -> float;
    fn pow(base: float, exp: float) -> float;
    fn abs(x: int) -> int;
    fn sin(x: float) -> float;
    fn cos(x: float) -> float;
}

fn main() {
    let kok = sqrt(64.0);
    let us = pow(2.0, 10.0);
    println("Karekök(64) =", kok); // 8.0
    println("2^10        =", us);  // 1024.0
}
```

* **Statik Tip Güvenliği:** `extern "C"` bildirimindeki parametre ve dönüş tipleri, derleme zamanında tip denetleyicisi tarafından katı bir şekilde doğrulanır.
* **Hibrit Çalışma:** Fonksiyonlar hem `oxide run` yorumlayıcısında hem de `oxide build` ile GCC `-O2` ve `-lm` (math library) kullanılarak doğrudan native makine kodunda çalıştırılır.

---

## 9. Karakter ve Bayt Tipleri (`char` ve `u8`)

Oxide, sistem programlama ve düşük seviyeli I/O işlemleri için Rust benzeri `char` ve `u8` ilkel tiplerini destekler:

```rust
fn main() {
    // 1. Karakter literalleri ('c')
    let ch: char = 'O';
    let ascii: int = to_int(ch); // 79
    let harf: char = to_char(ascii + 1); // 'P'

    // 2. Bayt literalleri (b'c') ve 8-bit tamsayılar
    let b1: u8 = b'A'; // 65
    let b2: u8 = 10;
    let b3: u8 = b1 + b2; // 75

    // 3. Karşılaştırma ve Desen Eşleştirme (match)
    if ch < 'Z' {
        println("ch Z'den önce gelir");
    }

    match ch {
        'A' => println("Harf A"),
        'O' => println("Harf O"),
        _   => println("Diger"),
    }
}
```

### Dahili Tip Dönüşüm Fonksiyonları:
* `to_float(n: int | float | u8) -> float`: Sayısal değeri 64-bit ondalıklı sayıya (`float`) dönüştürür.
* `to_int(c: char | u8 | int | float) -> int`: Karakterin, baytın veya ondalıklı sayının tamsayı değerini döner.
* `to_char(n: int | u8 | char) -> char`: Sayısal değeri karaktere dönüştürür.
* `to_u8(n: int | char | u8) -> u8` (veya `to_byte`): Değeri 8-bit işaretsiz bayta dönüştürür.

---

## 10. Referanslar, Borrow Checker ve Unsafe Bloklar

Oxide, bellek güvenliğini sıfır çalışma zamanı maliyetiyle derleme anında garanti etmek için Rust esintili **Borrow Checker** sistemine sahiptir:

```rust
extern "C" {
    fn sqrt(x: float) -> float;
}

fn main() {
    let mut x: int = 100;

    // 1. Paylaşılmış Referans (&T)
    let r1: &int = &x;
    let r2: &int = &x;
    println("r1:", *r1, "r2:", *r2); // 100, 100

    // 2. Non-Lexical Lifetimes (NLL)
    // r1 ve r2'nin son kullanımı yukarıda bittiği için ödünç serbest kalır.
    // Artık değiştirilebilir referans (&mut T) alınabilir:
    let r_mut: &mut int = &mut x;
    println("r_mut:", *r_mut);

    // 3. FFI Güvenlik Bariyeri (unsafe { ... })
    // Harici C fonksiyon çağrıları 'unsafe' blok içinde yapılmalıdır:
    unsafe {
        let kok = sqrt(64.0);
        println("Karekök:", kok);
    }
}
```

### Temel Borrow Kuralları:
1. Bir değişken için aynı anda **istendiği kadar paylaşılmış referans (`&T`)** VEYA **yalnızca tek bir değiştirilebilir referans (`&mut T`)** bulunabilir.
2. Bir değişken aktif olarak ödünç alınmışken doğrudan değiştirilemez (`x = 20` compile error).
3. Değişmez (`let`) bir değişken `&mut` ile ödünç alınamaz.
4. Harici C fonksiyon çağrıları (`extern "C"`) ve ham göstericiler (`*const T`, `*mut T`) `unsafe { ... }` blok gerektirir.

---

## 11. Standart Kütüphane Metotları ve Sistem API'si

### 11.1. Metin (String) Metotları:
```rust
let s = "  merhaba oxide  ";
let t = s.trim();                  // "merhaba oxide"
let u = t.to_upper();              // "MERHABA OXIDE"
let l = u.to_lower();              // "merhaba oxide"
let c = t.contains("oxide");       // true
let r = t.replace("oxide", "dünya"); // "merhaba dünya"
let ch = t.char_at(0);             // 'm' (char)
let parcalar = t.split(" ");       // ["merhaba", "oxide"] ([str])
let uzunluk = t.len();             // 13 (int)
```

### 11.2. Dizi (Array) Metotları:
```rust
let mut sayilar = [1, 2, 3];
sayilar.push(4);                   // Eleman ekler
let son = sayilar.pop();           // Son elemanı çıkarır (4)
let u = sayilar.len();             // Dizi uzunluğu (3)
```

### 11.3. Anahtar-Değer Haritası (`Map`):
```rust
let m = map_new();
m.set("kullanici", "can");
m.set("rol", "admin");

let ad = m.get("kullanici");       // "can"
let var_mi = m.has("rol");         // true
let adet = m.len();                // 2
let anahtarlar = m.keys();         // ["kullanici", "rol"]
```

### 11.4. Sistem API'si (CLI & Ortam Değişkenleri):
```rust
let cli_args = args();             // Komut satırı argümanları ([str])
let path = get_env("PATH");        // Ortam değişkenini oku (str)
// exit(0);                        // Programı çıkış kodu ile sonlandır
```

---

## 12. Anonim Fonksiyonlar (Closures) ve Fonksiyonel Metotlar

Oxide, Rust tarzı boru (`|...|`) sözdizimi ile birinci sınıf (first-class) anonim fonksiyonları (closures) destekler:

```rust
fn main() {
    // 1. Temel Closure Bildirimi ve Değişkene Atama
    let carp = |a: int, b: int| -> int { a * b };
    let sonuc = carp(6, 7); // 42

    // Kısa sözdizimi:
    let kare = |x: int| x * x;
    println("9'un karesi:", kare(9)); // 81

    // 2. Fonksiyonel Koleksiyon Metotları
    let sayilar = [1, 2, 3, 4, 5, 6];

    // .map() ile dönüştürme:
    let iki_kati = sayilar.map(|x: int| -> int { x * 2 });
    // [2, 4, 6, 8, 10, 12]

    // .filter() ile süzme:
    let ciftler = sayilar.filter(|x: int| -> bool { x % 2 == 0 });
    // [2, 4, 6]

    // 3. Bellek Arenası Kontrolü (arena_reset)
    // Ağır döngülerde veya geçici veri üretiminde bellek alanını anında sıfırlar:
    arena_reset();
}
```

---

## 13. Proje Yönetimi (`oxide.toml`) ve Test Motoru

Oxide, dahili bir proje yönetim ve test mekanizmasına sahiptir:

### 13.1. Yeni Proje Oluşturma
```bash
oxide new harika_proje
```
Bu komut aşağıdaki yapıyı üretir:
* `oxide.toml` (Paket manifestosu: isim, sürüm, giriş dosyası)
* `src/main.ox` (Ana program dosyası)
* `tests/test_basic.ox` (Birim test dosyası)

### 13.2. Dahili Test Motoru (`oxide test`)
Test dosyalarında `test_` ile başlayan fonksiyonlar otomatik olarak taranır ve çalıştırılır:
```rust
// tests/test_basic.ox
fn test_toplama() {
    assert(1 + 1 == 2);
}

fn test_metin() {
    assert("oxide".len() == 5);
}
```
Çalıştırmak için:
```bash
oxide test
# veya tek dosya testi:
oxide test tests/test_basic.ox
```

---

## 14. Generic Tipler ve Parametrik Polimorfizm (`<T>`)

Oxide, Rust tarzı parametrik polimorfizm ve sıfır maliyetli soyutlama (Zero-Cost Monomorphization) mimarisini destekler:

### 14.1. Generic Fonksiyonlar ve Tip Çıkarımı
```rust
fn kimlik<T>(x: T) -> T {
    x
}

fn ilkini_sec<A, B>(ilk: A, ikinci: B) -> A {
    ilk
}

fn main() {
    // 1. Otomatik Tip Çıkarımı:
    let n = kimlik(42);              // T = int
    let s = kimlik("merhaba");       // T = str
    let a = ilkini_sec(10, "ikinci"); // A = int, B = str

    // 2. Açık Tip Belirtimi (Turbofish ::<T>):
    let sayi = kimlik::<int>(100);
}
```

### 14.2. Generic Struct ve Impl Blokları
```rust
struct Kutu<T> {
    deger: T
}

struct Cift<T, U> {
    birinci: T,
    ikinci: U
}

impl<T> Kutu<T> {
    fn oku(self) -> T {
        self.deger
    }
}

fn main() {
    let k1 = Kutu { deger: 42 };
    let k2 = Kutu { deger: "oxide" };

    println("k1 oku:", k1.oku()); // 42
    println("k2 oku:", k2.oku()); // "oxide"

    let ikili = Cift { birinci: 10, ikinci: "puan" };
}
```

### 14.3. Sıfır Maliyetli Derleme (Monomorphization)
Derleyici, kodda kullanılan her somut tip için derleme zamanında özel fonksiyon ve struct tanımları üretir (`kimlik__int`, `kimlik__str`, `Kutu__int`, `Kutu__str`).
* Çalışma zamanında sanal metot tablosu (vtable), boxing veya `void*` gösterici ek yükü **yoktur**.
* Native C derlemesinde doğrudan C `struct` ve statik C fonksiyonlarına dönüştürülerek GCC tarafından makine koduna optimize edilir.

---

## 15. Hata Yönetimi (`Option<T>`, `Result<T, E>` ve `?` Operatörü)

Oxide, `null` veya kontrolsüz istisna (exception) kavramlarına sahip değildir. Bunun yerine Rust tarzı tip güvenli `Option` ve `Result` mimarisini kullanır:

### 15.1. `Option<T>` ile Değer Yokluğu (Null Güvenliği)
Bir değerin var olabileceğini veya olmayabileceğini ifade eder:
* `Some(deger)`: Değerin var olduğunu belirtir.
* `None`: Değerin olmadığını belirtir.

```rust
fn bol(pay: int, payda: int) -> Option<int> {
    if payda == 0 {
        return None;
    }
    return Some(pay / payda);
}

fn main() {
    let opt = bol(10, 2);

    if opt.is_some() {
        println("Sonuç:", opt.unwrap());
    }

    let bos: Option<int> = None;
    println("Varsayılanla aç:", bos.unwrap_or(0)); // 0
}
```

### 15.2. `Result<T, E>` ile Hata Yönetimi
İşlemlerin başarılı sonucunu (`Ok`) veya meydana gelen hatayı (`Err`) temsil eder:
* `Ok(deger)`: İşlemin başarılı olduğunu bildirir.
* `Err(hata)`: İşlemde hata oluştuğunu bildirir.

```rust
fn pozitif_bol(a: int, b: int) -> Result<int, str> {
    if b == 0 {
        return Err("Sıfıra bölme hatası!");
    }
    return Ok(a / b);
}

fn main() {
    let r = pozitif_bol(20, 4);
    println("Başarılı mı?:", r.is_ok());
    println("Değer:", r.unwrap()); // 5
}
```

### 15.3. `match` ile Desen Eşleme
`Some`, `None`, `Ok` ve `Err` varyantları `match` ifadesiyle değişken bağlanarak çözümlenebilir:
```rust
let sonuc: Result<int, str> = pozitif_bol(10, 0);

match sonuc {
    Ok(sayi) => {
        println("Başarı:", sayi);
    }
    Err(hata) => {
        println("Hata Mesajı:", hata);
    }
}
```

### 15.4. Hata Yayma: `?` (Try) Operatörü
Hata veya boşluk durumunda fonksiyonu erken sonlandırıp çağıran fonksiyona hatayı iletmek için `?` operatörü kullanılır:
```rust
fn zincir_hesap(a: int, b: int) -> Result<int, str> {
    // Eğer pozitif_bol Err dönerse, fonksiyondan hemen çıkılır ve Err döner.
    // Eğer Ok dönerse, içerisindeki değer açılır (unwrap).
    let ilk = pozitif_bol(a, b)?;
    let ikinci = pozitif_bol(ilk, 2)?;
    return Ok(ikinci + 10);
}

fn opt_hesap(a: int, b: int) -> Option<int> {
    // None dönerse fonksiyondan hemen None döner, Some ise açılır.
    let sonuc = bol(a, b)?;
    return Some(sonuc * 2);
}
```
* **Native C Optimizasyonu:** `?` operatörü C derleyicisinde GNU C Statement Expression `({ ... })` yapılarına dönüştürülerek sıfır ek yükle C seviyesinde erken dönüş sağlar.

---

## 16. Trait / Arayüz Sistemi (`trait` ve `impl Trait for Struct`)

Oxide, nesne yönelimli dillerdeki arayüzlere (interface) benzer ancak Rust gibi sınıfsız ve sıfır maliyetli soyutlama sağlayan **Trait** mekanizmasına sahiptir:

### 16.1. Trait Tanımlama ve Varsayılan Metotlar
Trait'ler fonksiyon imzaları içerebileceği gibi, varsayılan gövdeye sahip metotlar da içerebilir:

```rust
trait Goster {
    // Zorunlu metot: Trait'i uygulayan her struct bunu yazmalıdır
    fn goster(self) -> str;

    // Varsayılan metot: Struct bunu uygulamazsa otomatik devralınır
    fn ozet(self) -> str {
        return "Gosterilen: " + self.goster();
    }
}
```

### 16.2. Trait Uygulama (`impl Trait for Struct`)
Bir struct bir veya daha fazla trait uygulayabilir:

```rust
struct Nokta {
    x: int,
    y: int,
}

impl Goster for Nokta {
    fn goster(self) -> str {
        return "Nokta(" + self.x + ", " + self.y + ")";
    }
    // 'ozet' varsayılan olarak devralınır!
}

struct Dikdortgen {
    en: int,
    boy: int,
}

impl Goster for Dikdortgen {
    fn goster(self) -> str {
        return "Dikdortgen(" + self.en + "x" + self.boy + ")";
    }

    // Varsayılan metodu ezme (Override):
    fn ozet(self) -> str {
        return "Ozel Ozet -> " + self.goster();
    }
}
```

### 16.3. Trait Sınırları (Trait Bounds: `<T: Trait>`)
Generic fonksiyon ve yapılarda parametrenin belirli trait'leri uygulamasını şart koşabilirsiniz:

```rust
// Yalnızca 'Goster' trait'ini uygulayan tipler bu fonksiyona geçilebilir
fn ekrana_bas<T: Goster>(nesne: T) {
    println("Bilgi:", nesne.goster());
    println("Ozet:", nesne.ozet());
}

fn main() {
    let p = Nokta { x: 10, y: 20 };
    let d = Dikdortgen { en: 4, boy: 6 };

    ekrana_bas(p);
    ekrana_bas(d);
}
```

### 16.4. Sıfır Maliyetli Statik Gönderim (Static Dispatch)
Oxide derleyicisi trait çağrılarını monomorphization esnasında doğrudan somut struct metotlarına (`Nokta_goster`, `Dikdortgen_goster`) dönüştürür.
* **Sıfır Çalışma Zamanı Maliyeti:** Sanal metot tablosu (vtable), dinamik pointer takibi veya heap tahsisi yoktur.
* Native C derlemesinde doğrudan C fonksiyon çağrılarına dönüşerek GCC tarafından tam güçle optimize edilir.

---

## 17. Demetler ve Çoklu Dönüş Değerleri (Tuples & Destructuring)

Oxide, farklı tiplerdeki değerleri tek bir bileşik değerde toplamak ve fonksiyonlardan birden fazla değer döndürmek için Rust tarzı demetleri (`tuples`) destekler:

### 17.1. Demet Tanımlama ve Eleman Erişimi
```rust
// Farklı tiplerde elemanlar içeren bir demet:
let t = (42, "oxide", true);

// İndeks tabanlı nokta erişimi (0'dan başlar):
println(t.0); // 42
println(t.1); // "oxide"
println(t.2); // true

// Demetin tamamını yazdırma:
println(t);   // (42, oxide, true)
```

### 17.2. Değiştirilebilir (Mutable) Demet Elemanları
```rust
let mut mt = (10, 20);
mt.0 = 99;
println(mt.0, mt.1); // 99 20
```

### 17.3. Fonksiyonlardan Çoklu Değer Dönüşü ve Ayrıştırma (Destructuring)
```rust
fn bolme_ve_kalan(bolunen: int, bolen: int) -> (int, int) {
    let bolum = bolunen / bolen;
    let kalan = bolunen % bolen;
    return (bolum, kalan);
}

fn main() {
    // Demeti ayrıştırarak yerel değişkenlere bağlama:
    let (b, k) = bolme_ve_kalan(29, 6);
    println("Bölüm:", b, "Kalan:", k); // Bölüm: 4 Kalan: 5

    // Ayrıştırmada içsel değiştirilebilirlik (mut x, y):
    let (mut sayac, artis) = (0, 5);
    sayac = sayac + artis;
}
```

### 17.4. Demetler ile Desen Eşleme (Pattern Matching)
```rust
let nokta = (1, 2);
let sonuc = match nokta {
    (0, 0) => "Orijin noktası",
    (1, 2) => "Hedef koordinat",
    (x, _) => "X ekseninde özel nokta",
    _ => "Diğer",
};
```

### 17.5. Sıfır Ek Yük ve Native C Entegrasyonu
* Her özgün `(T1, T2, ...)` demet tipi, Native C transpiler aşamasında otomatik olarak bir C `typedef struct { T1 _0; T2 _1; ... } _OxTuple_...;` yapısına dönüştürülür.
* Heap tahsisi (malloc/free) veya kutulama (boxing) yoktur; doğrudan yığında (stack) veya CPU register'larında tutularak GCC `-O2` tarafından optimize edilir.

---

## 18. String Formatlama & Metin İnterpolasyonu (`f"..."` ve `format(...)`)

Oxide, metinleri birleştirmek için Python ve modern Rust dillerinden ilham alan zengin ve tip güvenli metin interpolasyonu ve formatlama araçları sunar:

### 18.1. `f"..."` Sözdizimi ile İnterpolasyon
Metin dizgisinin başına `f` harfi getirilerek içeride `{ifade}` süslü parantezleri ile değişkenler ve ifadeler doğrudan yerleştirilebilir:

```rust
let ad = "Can";
let skor = 95;
let mesaj = f"Merhaba {ad}, basari puanin: {skor}!";
println(mesaj);
```

### 18.2. Süslü Parantez İçinde İfade (Expression) Değerlendirme
Süslü parantez içine yalnızca düz değişkenler değil, her türlü geçerli Oxide ifadesi ve metot çağrısı yazılabilir:

```rust
let x = 10;
let y = 20;
let bilgi = f"{x} + {y} = {x + y}, {x} * {y} = {x * y}";
println(bilgi);

let metin = "sistem";
println(f"Buyuk harf: {metin.to_upper()}");
```

### 18.3. Çoklu Tip Desteği
İnterpolasyon ifadeleri otomatik olarak tüm Oxide tiplerini destekler:
* Tamsayılar (`int`) ve Ondalıklı sayılar (`float`)
* Mantıksal değerler (`bool` -> `"true"` / `"false"`)
* Karakterler (`char`) ve Baytlar (`u8`)
* Seçenekler (`Option<T>` -> `"Some(42)"` / `"None"`)
* Sonuçlar (`Result<T, E>` -> `"Ok(100)"` / `"Err(...)"`)
* Demetler (`(int, str, bool)` -> `"(1, test, true)"`)

```rust
let opt = Some(42);
let demet = (10, "basarili", true);
println(f"Option: {opt}, Demet: {demet}");
```

### 18.4. Süslü Parantez Kaçışı (`{{` ve `}}`)
Format metinlerinde düz süslü parantez karakteri (`{` veya `}`) bırakmak için çift parantez (`{{` veya `}}`) kullanılır:

```rust
let sayi = 42;
let json = f"{{\"deger\": {sayi}}}";
println(json); // {"deger": 42}
```

### 18.5. Rust Tarzı `format("...", args...)`
Rust tarzı şablon yer tutucuları (`{}`) ile formatlama da desteklenir:

```rust
let isim = "Demir";
let puan = 1500;
let s = format("Oyuncu: {}, Puan: {}", isim, puan);
println(s);
```

### 18.6. Bellek Verimliliği ve Native C Performansı
* Formatlanan metin parçaları, Oxide'ın yüksek performanslı thread-local ring buffer arenasında (`_ox_arena_alloc`) oluşturulur ve birleştirilir.
* Dinamik `malloc`/`free` maliyeti yoktur, bellek sızıntısı (memory leak) riski sıfırdır.
* Native C derleyicisi (`oxide build`), `_ox_to_str` C11 `_Generic` makrosu ile compile-time'da en verimli dönüştürme fonksiyonunu seçer.

---

## 19. Dilimler ve Aralık İndeksleme (Slices & Range Indexing: `&[T]`, `&str`)

Oxide, Rust tarzı **sıfır kopyalama (zero-copy)** dilimleme sistemine sahiptir. Dizilerin veya metinlerin tamamını veya belirli bir aralığını kopyalamadan, yalnızca bir bellek işaretçisi ve uzunluk (`ptr + len`) içeren hafif bir görünüm (view) olarak temsil eder.

### 19.1. Aralık İndeksleme Sözdizimi
Diziler ve metinler üzerinde `[baslangic..bitis]` aralık sözdizimi ile dilim oluşturulur:

* `arr[bas..son]`: `bas` indeksinden başlayıp `son` indeksine kadar (dahil değil, yarı açık aralık `[bas, son)`) elemanları içeren dilim.
* `arr[bas..]`: `bas` indeksinden dizinin sonuna kadar olan dilim.
* `arr[..son]`: Dizinin başından (indeks 0) `son` indeksine kadar olan dilim.
* `arr[..]`: Dizinin tüm elemanlarını içeren tam dilim.

```rust
let sayilar = [10, 20, 30, 40, 50, 60];

let d1 = sayilar[1..4];  // [20, 30, 40]
let d2 = sayilar[3..];   // [40, 50, 60]
let d3 = sayilar[..3];   // [10, 20, 30]
let d4 = sayilar[..];    // [10, 20, 30, 40, 50, 60]
```

### 19.2. Dilim İndeksleme ve Alt Dilimleme (Sub-slicing)
Dilimler üzerinde indeksleme (`d[0]`), uzunluk sorgulama (`len(d)` veya `d.len()`) ve tekrar dilimleme yapılabilir:

```rust
println(d1[0]); // 20
println(d1[1]); // 30
println(d1.len()); // 3

let alt = d1[1..3]; // [30, 40]
```

### 19.3. Fonksiyonlara Dilim Geçirme ve Otomatik Zorlama (Coercion)
Fonksiyonların parametre olarak `&[T]` veya `&str` alması, hem tam dizilerin hem de alt dilimlerin gereksiz kopya oluşturulmadan fonksiyona aktarılmasını sağlar:

```rust
fn dizi_toplami(dilim: &[int]) -> int {
    let mut top = 0;
    for x in dilim {
        top = top + x;
    }
    top
}

let sayilar = [10, 20, 30, 40, 50, 60];
// Dizi doğrudan &[int] bekleyen fonksiyona verilebilir:
let tum_toplam = dizi_toplami(sayilar); // 210

// Veya alt dilim verilebilir:
let dilim_toplami = dizi_toplami(sayilar[1..4]); // 90
```

### 19.4. Dilimi Dinamik Diziye Dönüştürme (`to_array()`)
Dilimler salt-okunur görünümlerdir. Dilimden bağımsız, eleman eklenebilir yeni bir dinamik dizi oluşturmak için `.to_array()` metodu kullanılır:

```rust
let yeni_dizi = d1.to_array();
push(yeni_dizi, 999);
println(yeni_dizi); // [20, 30, 40, 999]
println(d1);        // [20, 30, 40] (orijinal dilim etkilenmez)
```

### 19.5. Metin Dilimleri (`&str`)
Aynı aralık sözdizimi metinler (`str`) üzerinde de çalışır ve `&str` döner:

```rust
let metin = "merhaba oxide";
let kelime1: &str = metin[0..7]; // "merhaba"
let kelime2: &str = metin[8..];  // "oxide"

println(f"Dilim: {kelime1}, Uzunluk: {kelime1.len()}");
println(kelime2.to_upper()); // "OXIDE"
```

### 19.6. C11 Transpiler & Bellek Güvenliği
* C tarafında `_OxSlice` (`int64_t* ptr`, `int64_t len`) ve `_OxStrSlice` (`const char* ptr`, `int64_t len`) struct'ları kullanılır.
* Slicing işlemleri `O(1)` sürede gerçekleşir ve sıfır bellek tahsisi (zero allocation) gerektirir.
* Sınır aşımı durumlarında (`start > len`, `start > end`) güvenli çalışma zamanı panik mesajı üretilir.

---

## 20. Operatör Aşırı Yükleme ve Standart Traitler (Operator Overloading)

Oxide, Rust tarzı operatör aşırı yükleme mimarisini destekler. Kullanıcı tanımlı `struct` tipleri için yerleşik veya özel trait'ler (`Add`, `Sub`, `Mul`, `Div`, `Neg`, `Eq`, `Index`) uygulanarak standart dil operatörleri doğrudan nesneler üzerinde kullanılabilir.

### 20.1. Desteklenen Operatörler ve Metot Eşleşmeleri

| Operatör | Standart Trait | İlgili Metot İmzası | Açıklama |
| :--- | :--- | :--- | :--- |
| `a + b` | `Add` | `fn add(self, other: Rhs) -> Output` | Toplama operatörü |
| `a - b` | `Sub` | `fn sub(self, other: Rhs) -> Output` | Çıkarma operatörü |
| `a * b` | `Mul` | `fn mul(self, other: Rhs) -> Output` | Çarpma operatörü |
| `a / b` | `Div` | `fn div(self, other: Rhs) -> Output` | Bölme operatörü |
| `a % b` | `Rem` | `fn rem(self, other: Rhs) -> Output` | Mod operatörü |
| `-a` | `Neg` | `fn neg(self) -> Output` | Tek terimli eksi (işaret tersleme) |
| `!a` | `Not` | `fn not(self) -> Output` | Mantıksal / bitsel değil |
| `a == b` | `Eq` | `fn eq(self, other: Rhs) -> bool` | Eşitlik karşılaştırması |
| `a != b` | `Eq` / `Ne` | `fn ne` veya otomatik `!eq` | Eşitsizlik karşılaştırması |
| `a[i]` | `Index` | `fn index(self, idx: Idx) -> Output` | İndeks ile eleman okuma |
| `a[i] = v`| `Index` | `fn index_set(mut self, idx: Idx, val: Val)` | İndeks ile eleman atama |

### 20.2. Örnek: `Vector2D` Vektör Matematiği

```rust
struct Vector2D {
    x: int,
    y: int,
}

impl Add for Vector2D {
    fn add(self, other: Vector2D) -> Vector2D {
        Vector2D { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Mul for Vector2D {
    fn mul(self, factor: int) -> Vector2D {
        Vector2D { x: self.x * factor, y: self.y * factor }
    }
}

impl Neg for Vector2D {
    fn neg(self) -> Vector2D {
        Vector2D { x: -self.x, y: -self.y }
    }
}

impl Eq for Vector2D {
    fn eq(self, other: Vector2D) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Index for Vector2D {
    fn index(self, idx: int) -> int {
        if idx == 0 { self.x } else { self.y }
    }
}
```

### 20.3. Doğal Kullanım ve İfade Zincirleme (Chaining)

Operatörler tamsayı veya float ifadeler gibi doğal bir şekilde birleştirilebilir:

```rust
let v1 = Vector2D { x: 10, y: 20 };
let v2 = Vector2D { x: 3, y: 7 };

let v3 = (v1 + v2) * 2;  // Vector2D(26, 54)
let v4 = -v1;            // Vector2D(-10, -20)
let esit_mi = (v1 == v2); // false
let ilk = v1[0];          // 10
```

### 20.4. Trait Sınırları (Trait Bounds) ile Generic Fonksiyonlar

Operatör trait'leri, generic fonksiyon parametrelerinde kısıtlayıcı sınır (`<T: Add>`) olarak kullanılabilir:

```rust
fn topla_generic<T: Add>(a: T, b: T) -> T {
    a.add(b)
}
```

### 20.5. Sıfır Ek Yük ve Native GCC Derlemesi

* Monomorphizer ve C transpiler, operatör aşırı yüklemelerini doğrudan C struct fonksiyon çağrılarına (`Vector2D_add(&(v1), v2)`) indirger.
* Karmaşık zincirlemelerde (`(v1 + v2) * 2`) GNU C Statement Expressions (`({ ... })`) kullanılarak gereksiz bellek kopyalamaları engellenir ve GCC `-O2` optimizasyonu ile en yüksek çalışma hızı elde edilir.

---

## 21. Paket Yönetimi, `target/` Mimarisi, `oxide.lock` ve `clean`

Oxide, tıpkı Rust/Cargo felsefesinde olduğu gibi proje dizinini temiz tutan profesyonel bir derleme ve kilit dosyası yönetimine sahiptir.

### 21.1. Yeni Proje Başlatma (`oxide new`)

```bash
oxide new sepet_uygulamasi
```

Bu komut aşağıdaki dizin yapısını otomatik oluşturur:
```text
sepet_uygulamasi/
├── oxide.toml         # Proje metaverileri (isim, versiyon, giriş dosyası)
├── oxide.lock         # Kilit dosyası (otomatik üretilir)
├── .gitignore         # /target/ klasörünü yoksayar
├── src/
│   └── main.ox        # Uygulama giriş noktası
└── tests/
    └── test_basic.ox  # Birim test şablonu
```

### 21.2. `target/` Dizin Mimarisi

`oxide build` çalıştırıldığında üretilen ara `.c` dosyaları ve derlenmiş native binary çıktıları kaynak kod klasörünü kirletmez:

* **Debug Profili (`oxide build`):**
  * Binary: `target/debug/<proje_adi>`
  * Üretilen C kodu: `target/debug/c_gen/<proje_adi>.c`
* **Release Profili (`oxide build --release`):**
  * Binary: `target/release/<proje_adi>` (GCC `-O3` optimizasyonu)
  * Üretilen C kodu: `target/release/c_gen/<proje_adi>.c`
* **Otomatik `.gitignore`:** `target/` klasörü altına `*` içeren bir `.gitignore` yerleştirilerek git kirliliği tamamen engellenir.

### 21.3. `oxide.lock` Kilit Dosyası

Projenin derleme durumu, sürüm bilgileri ve yerel paket metaverileri `oxide.lock` dosyasına otomatik kaydedilir:
```toml
# Bu dosya Oxide tarafından otomatik üretilmiştir.
# El ile düzenlenmesi önerilmez.
version = 1

[[package]]
name = "sepet_uygulamasi"
version = "0.1.0"
entry = "src/main.ox"
source = "local"
```

### 21.4. `oxide clean` Komutu

Tüm derleme çıktılarını, ara C dosyalarını ve binary'leri tek komutla temizler:
```bash
oxide clean
# ✔ 'target/' dizini başarıyla temizlendi.
```

---

## 22. Akıllı İşaretçiler & Özyinelemeli Veri Yapıları (`Box<T>`, `Rc<T>`, `RefCell<T>`)

Oxide, Rust'ın bellek güvenliği ve modern sistem programlama yaklaşımlarını yansıtan güçlü akıllı işaretçi (smart pointer) tiplerine sahiptir.

### 22.1. `Box<T>`: Yığın Tahsisi ve Özyinelemeli Veri Yapıları
`Box<T>`, veriyi yığında (heap / arena) tutan tekil sahipliğe sahip bir işaretçidir. Boyutu derleme zamanında bilinemeyen özyinelemeli (recursive) tipler (Bağlı Liste, Ağaç vb.) oluşturmak için esastır.

```rust
struct Node {
    val: int,
    next: Option<Box<Node>>,
}

fn main() {
    // Box oluşturma
    let b: Box<int> = Box::new(42);
    
    // Dereferans okuma (*b)
    println("Box degeri:", *b);

    // Dereferans üzerinden yazma (*b = yeni_deger)
    let mut b_mut = Box::new(100);
    *b_mut = 250;

    // Özyinelemeli Bağlı Liste (Recursive LinkedList)
    let n3 = Node { val: 30, next: None };
    let n2 = Node { val: 20, next: Some(Box::new(n3)) };
    let n1 = Node { val: 10, next: Some(Box::new(n2)) };

    // Otomatik dereferans (Auto-deref) ile alan erişimi
    match n1.next {
        Some(dugum) => {
            println("İkinci düğüm değeri:", dugum.val); // dugum.val doğrudan çalışır
        }
        None => {}
    }
}
```

### 22.2. `Rc<T>`: Referans Sayımlı Paylaşımlı Bellek (Reference Counting)
Tek bir veriye birden fazla sahip (ownership) gerektiren durumlarda kullanılır.
* `Rc::new(deger)`: Yeni bir referans sayımlı işaretçi oluşturur (sayac = 1).
* `rc.clone()`: Veriyi kopyalamaz; yalnızca referans sayacını 1 artırır.
* `Rc::strong_count(rc)`: Anlık referans sayısını döner.
* `*rc`: İçerideki değere güvenli okuma erişimi sağlar.

```rust
let rc1: Rc<int> = Rc::new(777);
println("Sayaç:", Rc::strong_count(rc1)); // 1

let rc2 = rc1.clone();
println("Sayaç (klon sonrası):", Rc::strong_count(rc1)); // 2
println("rc2 üzerinden okuma:", *rc2); // 777
```

### 22.3. `RefCell<T>`: İçsel Değiştirilebilirlik (Interior Mutability)
Değiştirilemez (immutable) görünen bir referans üzerinden içerideki veriyi kontrollü şekilde değiştirmeyi sağlar.
* `RefCell::new(deger)`: Hücre oluşturur.
* `cell.borrow()`: İçerideki değeri okumak için referans döner (`*cell.borrow()`).
* `cell.borrow_mut()`: İçerideki değeri güncellemek için referans döner (`*cell.borrow_mut() = yeni`).

```rust
let cell = RefCell::new(50);
println("İlk değer:", *cell.borrow());

*cell.borrow_mut() = 999;
println("Güncel değer:", *cell.borrow()); // 999
```

### 22.4. Native C Kod Üretimi & Sıfır Maliyetli İleri Bildirimler (Forward Declarations)
* `Box<T>` ve `RefCell<T>`, native C kodunda doğrudan optimize pointer `T*` olarak derlenir.
* `Rc<T>`, runtime arena destekli `_OxRc*` yapısı ve atomik/hızlı referans artırımı ile çalışır.
* Özyinelemeli `struct Node { next: Option<Box<Node>> }` yapıları için C derleyicisinde döngüsel bağımlılık hatası oluşmaması amacıyla tüm struct tipleri `typedef struct Node Node;` şeklinde önceden bildirilir (Forward Declarations).
* Hem yorumlayıcı hem de GCC `-O2` çıktısı %100 birebir uyumludur.

---

## 23. Derleme Zamanı Sabitleri (`const`) & Standart Dosya/Dizin Sistemi (`fs::`, `env::`)

Oxide, derleme anında çözülen değişmez sabitler ile modern sistem programlamanın temel ihtiyacı olan güvenli dosya ve ortam etkileşim API'lerine sahiptir.

### 23.1. `const` Sabitleri (Compile-Time Constants)
`const` ifadeleri derleme zamanında çözümlenir (compile-time evaluation) ve program boyunca hiçbir çalışma zamanı ek yükü olmadan inlined edilir.
* Aritmetik (`+`, `-`, `*`, `/`, `%`), mantıksal (`!`, `&&`, `||`), karşılaştırma ve metin birleştirme ifadelerini derleme anında hesaplar.
* Hem dosya seviyesinde (global) hem de fonksiyon/blok içinde tanımlanabilir.

```rust
// Global derleme zamanı sabitleri
const BUFFER_SIZE: int = 1024 * 4;
const PI: float = 3.14159;
const APP_NAME: str = "OxideFS";
const GREETING: str = APP_NAME + " Sistem Araci";

fn main() {
    // Fonksiyon içi yerel sabit
    const CARPAN: int = 10 + 2;
    let sonuc = BUFFER_SIZE / CARPAN; // 341
    println("Hesaplanan:", sonuc);
}
```

### 23.2. Dosya Sistemi API'si (`fs::`)
Rust'ın `std::fs` tasarımına paralel olarak dosya işlemleri güvenli `Result` ve `Option` tipleriyle yönetilir:
* `fs::write(yol, icerik) -> Result<void, str>`: Dosyaya içerik yazar.
* `fs::read_to_string(yol) -> Result<str, str>`: Dosya içeriğini metin olarak okur.
* `fs::exists(yol) -> bool`: Dosyanın veya dizinin var olup olmadığını kontrol eder.
* `fs::remove_file(yol) -> Result<void, str>`: Belirtilen dosyayı siler.
* `fs::create_dir(yol) -> Result<void, str>`: Yeni bir dizin oluşturur.

```rust
let yol = "veri.txt";

// Dosya yazma
let yaz_sonuc = fs::write(yol, "Oxide dosya sistemi testi");
if yaz_sonuc.is_ok() {
    println("Dosya başarıyla yazıldı!");
}

// Dosya okuma & Desen eşleme
match fs::read_to_string(yol) {
    Ok(icerik) => println("İçerik:", icerik),
    Err(hata) => println("Okuma hatası:", hata),
}

// Varlık kontrolü ve silme
if fs::exists(yol) {
    fs::remove_file(yol);
}
```

### 23.3. Sistem ve Çevre Değişkenleri API'si (`env::`)
* `env::args() -> [str]`: Komut satırı argümanlarını dinamik dizi olarak döner.
* `env::var(anahtar) -> Option<str>`: Sistem çevre değişkenini okur; değişken tanımlıysa `Some(deger)`, yoksa `None` döner.
* `env::current_dir() -> Result<str, str>`: Uygulamanın aktif çalışma dizinini döner.

```rust
// Komut satırı argümanları
let argumanlar = env::args();
println("Argüman sayısı:", len(argumanlar));

// Çevre değişkeni kontrolü
match env::var("PATH") {
    Some(yol) => println("PATH bulundu"),
    None => println("PATH bulunamadı"),
}

// Çalışma dizini
let pwd = env::current_dir()?;
println("Aktif dizin:", pwd);
```

### 23.4. Native C Kod Üretimi & GCC Optimizasyonu
* `const` ifadeleri C kodunda GNU C `#define` sabitleri olarak üretilir ve tam tip güvenliği ile inlined edilir.
* Dosya ve ortam API'leri C11 / POSIX standart kütüphanesine (`fopen`, `fread`, `fwrite`, `fclose`, `remove`, `access`, `mkdir`, `getenv`, `getcwd`) doğrudan transpile edilir.
* C seviyesinde `int main(int argc, char** argv)` argümanları otomatik yakalanıp `env::args()` fonksiyonuna aktarılır.
* Yorumlayıcı (`oxide run`) ve Native Binary (`oxide build`) arasında %100 birebir davranış ve çıktı uyumu sağlanır.














