// Oxide Dili - Aşama 14: Çoklu Dönüş Değerleri ve Demetler (Tuples) Demo

fn koordinat_ve_skor() -> (int, int, str) {
    return (10, 25, "Aktif Oyuncu");
}

fn bolme_ve_kalan(bolunen: int, bolen: int) -> (int, int) {
    let bolum = bolunen / bolen;
    let kalan = bolunen % bolen;
    return (bolum, kalan);
}

fn main() {
    println("=== Oxide Çoklu Dönüş Değerleri ve Demetler (Aşama 14) ===");

    // 1. Temel Demet (Tuple) Tanımlama ve İndeksleme
    let t = (42, "oxide sistem dili", true);
    println("t.0 (int):", t.0);
    println("t.1 (str):", t.1);
    println("t.2 (bool):", t.2);
    println("Demetin kendisi (println):", t);

    // 2. Değiştirilebilir (Mutable) Demet Elemanı Güncelleme
    let mut mt = (100, 200);
    println("mt ilk hali:", mt.0, mt.1);
    mt.0 = 555;
    mt.1 = 999;
    println("mt guncel hali:", mt.0, mt.1);

    // 3. Fonksiyondan Çoklu Değer Dönüşü ve Ayrıştırma (Destructuring)
    let (x, y, unvan) = koordinat_ve_skor();
    println("Ayrıştırılan değerler -> x:", x, "y:", y, "unvan:", unvan);

    let (bolum, kalan) = bolme_ve_kalan(29, 6);
    println("29 / 6 -> Bölüm:", bolum, "Kalan:", kalan);

    // 4. Ayrıştırmada İçsel Mutability (mut x, y)
    let (mut sayac, carpici) = (5, 10);
    sayac = sayac + 15;
    println("sayac:", sayac, "carpici:", carpici, "carpim:", sayac * carpici);

    // 5. Demetler ile Pattern Matching (Desen Eşleme)
    let ikili = (1, 2);
    let mesaj = match ikili {
        (0, 0) => "Orijin noktasi",
        (1, 2) => "Hedef koordinat (1, 2) yakalandi!",
        _ => "Diger koordinat",
    };
    println("Match sonucu:", mesaj);

    let kontrol = (0, "hata");
    let sonuc_metni = match kontrol {
        (0, msg) => msg,
        (_, _) => "basarili",
    };
    println("Kontrol sonucu:", sonuc_metni);

    println("=== Tum Demet (Tuple) Testleri Basariyla Tamamlandi! ===");
}
