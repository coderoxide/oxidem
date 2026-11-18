fn test_basic_closure() -> void {
    println("=== 1. Temel Closure ve Degiskene Atama ===");
    let carp = |x: int, y: int| -> int {
        x * y
    };
    let sonuc = carp(6, 7);
    println("carp(6, 7) sonucu:", sonuc);

    let karesi = |x: int| x * x;
    println("karesi(9):", karesi(9));
}

fn test_functional_methods() -> void {
    println("=== 2. Fonksiyonel Dizi Metotlari (map, filter) ===");
    let sayilar = [1, 2, 3, 4, 5, 6];

    // map ile her elemanin 2 katini alma
    let iki_kat = sayilar.map(|x: int| x * 2);
    println("sayilar.map(|x| x * 2):", iki_kat);

    // filter ile sadece cift sayilari filtreleme
    let ciftler = sayilar.filter(|x: int| x % 2 == 0);
    println("sayilar.filter(|x| cift):", ciftler);
}

fn test_arena_reset() -> void {
    println("=== 3. Bellek Arenasi Resetleme (arena_reset) ===");
    let mut i = 0;
    while i < 3 {
        let gecici = [i, i + 1, i + 2];
        println("Dongu adimi:", i, "Gecici dizi:", gecici);
        arena_reset();
        i = i + 1;
    }
    println("Arena basariyla sifirlandi!");
}

fn main() -> void {
    println("=== Oxide Closures & Functional Methods (Aşama 10) ===");
    test_basic_closure();
    test_functional_methods();
    test_arena_reset();
    println("=== Tum Aşama 10 Testleri Basariyla Tamamlandi! ===");
}
