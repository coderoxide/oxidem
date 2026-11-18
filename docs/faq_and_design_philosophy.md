# Oxide Tasarım Felsefesi ve SSS (FAQ & Philosophy)

---

## 1. Neden Oxide?

Oxide, modern sistem programlama dillerinin gücünü ve güvenliğini, öğrenmesi ve geliştirmesi kolay minimalist bir dille birleştirmek amacıyla tasarlanmıştır.

### Rust'tan Alınan İlkeler
* **Değişmezlik Varsayılan Olmalıdır (Immutability by Default):** Değişkenler varsayılan olarak sabittir (`let`). Değiştirmek isteyen geliştirici niyetini açıkça belirtmelidir (`let mut`).
* **İfade Odaklılık (Expression-Oriented):** Bloklar, `if` ve `match` yapıları birer deyim değil, değer üreten ifadelerdir. Bu, geçici değişken kirliliğini ortadan kaldırır.
* **Veri ve Davranışın Ayrılması:** Nesne yönelimli sınıflar (`class`) yerine veriyi tanımlayan `struct` ve davranışları tanımlayan `impl` blokları kullanılır.
* **Cebirsel Tipler ve Desen Eşleştirme:** `enum` ve `match` ile tüm durumların açık ve güvenli şekilde ele alınması sağlanır.

---

## 2. Neden C Transpiler Tercih Edildi?

Doğrudan LLVM backend yazmak yerine C11 Transpiler tercih edilmesinin temel nedenleri:

1. **Taşınabilirlik (Portability):** C derleyicisi (GCC/Clang/MSVC) bulunan her mimaride (x86_64, ARM, RISC-V, gömülü sistemler) Oxide kodları anında derlenebilir.
2. **Optimizasyon:** GCC ve Clang'in 40 yıllık derleyici optimizasyonlarından (`-O2`, `-O3`, LTO, vektörizasyon) sıfır ek maliyetle yararlanılır.
3. **Şeffaflık:** Üretilen C kodu geliştiriciler tarafından doğrudan incelenebilir ve gerektiğinde C kütüphaneleriyle entegre edilebilir.

---

## 3. Sıkça Sorulan Sorular (SSS)

### S: Oxide kodu çalıştırmak için sistemde ne kurulu olmalı?
* **Yorumlayıcı modu (`run` ve `repl`):** Yalnızca Rust/Cargo.
* **Native derleme modu (`build`):** Rust/Cargo ve sistemde `gcc` veya `clang`.

### S: Oxide'da bellek yönetimi nasıl çalışıyor?
* **Yorumlayıcıda (Tree-walk):** Rust'ın kendi referans sayımı (`Rc<RefCell<...>>`) mekanizmasıyla yönetilir.
* **Native C Derlemede:** Statik ve yığın (stack) tabanlı tahsisatlar kullanılır. Dinamik diziler ve metinler için standart heap yönetimi (`malloc`/`free`) uygulanır.

### S: Garbage Collector (GC) var mı?
Hayır. Tıpkı Rust gibi Oxide da bir çöp toplayıcı (Garbage Collector) çalışma zamanı içermez. Bu sayede öngörülebilir gecikme (predictable latency) ve minimum bellek ayak izi sağlar.

### S: Sıradaki büyük özellikler neler?
* Statik Tip Denetleyici (Semantic Type Checker)
* Modül ve paketleme sistemi (`import "matematik.ox"`)
* C kütüphanelerini doğrudan çağırabilen FFI (Foreign Function Interface) katmanı.
