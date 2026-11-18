fn main() {
    println("=== Oxide Dizi Testleri ===");

    let mut sayilar = [10, 20, 30, 40];
    println("Başlangıç dizisi:", sayilar);
    println("Dizinin uzunluğu:", len(sayilar));

    // Eleman okuma
    println("İlk eleman (sayilar[0]):", sayilar[0]);
    println("Üçüncü eleman (sayilar[2]):", sayilar[2]);

    // Eleman değiştirme
    sayilar[1] = 99;
    println("sayilar[1] = 99 sonrası:", sayilar);

    // push ile yeni eleman ekleme
    push(sayilar, 500);
    println("push(500) sonrası:", sayilar);
    println("Yeni uzunluk:", len(sayilar));

    // pop ile son elemanı çıkarma
    let son = pop(sayilar);
    println("pop ile çıkarılan eleman:", son);
    println("Kalan dizi:", sayilar);

    // Döngü ile elemanları yazdırma
    println("\n--- Döngü ile Dizi Elemanları ---");
    let mut i = 0;
    while i < len(sayilar) {
        println("Eleman [" + i + "] =", sayilar[i]);
        i = i + 1;
    }
}
