// Oxide Generics & Parametric Polymorphism Demo (Aşama 11)

// 1. Tek tip parametreli generic fonksiyon
fn kimlik<T>(x: T) -> T {
    x
}

// 2. İki farklı tip parametreli generic fonksiyon
fn ilkini_sec<A, B>(ilk: A, ikinci: B) -> A {
    ilk
}

// 3. Generic Struct Yapısı
struct Kutu<T> {
    deger: T
}

// 4. İki parametreli Generic Struct
struct Cift<T, U> {
    birinci: T,
    ikinci: U
}

// 5. Generic Struct için Impl Metotları
impl<T> Kutu<T> {
    fn oku(self) -> T {
        self.deger
    }
}

fn main() {
    println("=== Oxide Generic Tipler (Aşama 11) ===");

    // 1. Fonksiyon Tipi Çıkarımı (Type Inference)
    let sayi = kimlik(42);
    let metin = kimlik("merhaba oxide");
    let ondalik = kimlik(3.14);
    let bayt = kimlik(b'Z');

    println("kimlik(42):", sayi);
    println("kimlik('merhaba oxide'):", metin);
    println("kimlik(3.14):", ondalik);
    println("kimlik(b'Z'):", bayt);

    // 2. Çoklu Generic Parametre
    let secim1 = ilkini_sec(100, "ikinci");
    let secim2 = ilkini_sec("oncelikli", 999);
    println("ilkini_sec(100, 'ikinci'):", secim1);
    println("ilkini_sec('oncelikli', 999):", secim2);

    // 3. Açık Tip Belirtimi (Turbofish: fn::<T>)
    let acik_sayi = kimlik::<int>(99);
    let acik_metin = kimlik::<str>("turbofish calisiyor");
    println("acik_sayi (::<int>):", acik_sayi);
    println("acik_metin (::<str>):", acik_metin);

    // 4. Generic Struct Başlatma ve Alan Erişimi
    let k1 = Kutu { deger: 555 };
    let k2 = Kutu { deger: "kutu icerigi" };
    println("k1.deger (int):", k1.deger);
    println("k2.deger (str):", k2.deger);

    // 5. Generic Metot Çağrısı (impl<T> Kutu<T>)
    let k1_deger = k1.oku();
    let k2_deger = k2.oku();
    println("k1.oku():", k1_deger);
    println("k2.oku():", k2_deger);

    // 6. Çok Tipli Struct (Cift<T, U>)
    let ikili = Cift { birinci: 10, ikinci: "skor" };
    println("ikili.birinci:", ikili.birinci, "ikili.ikinci:", ikili.ikinci);

    // 7. Doğrulama (Asserts)
    assert(sayi == 42);
    assert(metin == "merhaba oxide");
    assert(k1.deger == 555);
    assert(k1_deger == 555);
    assert(k2_deger == "kutu icerigi");
    assert(ikili.birinci == 10);

    println("=== Tum Generic Testleri Basariyla Tamamlandi! ===");
}
