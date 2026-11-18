// Oxide Dilimler ve Aralık İndeksleme Demo (Aşama 16)

fn dizi_toplami(dilim: &[int]) -> int {
    let mut toplam = 0;
    for eleman in dilim {
        toplam = toplam + eleman;
    }
    return toplam;
}

fn main() {
    println("=== Oxide Dilimler ve Aralık İndeksleme (Aşama 16) ===");

    // 1. Dizi Dilimleme (Array Slices)
    let sayilar = [10, 20, 30, 40, 50, 60];
    let d1 = sayilar[1..4]; // [20, 30, 40]
    let d2 = sayilar[3..];  // [40, 50, 60]
    let d3 = sayilar[..3];  // [10, 20, 30]
    let d4 = sayilar[..];   // [10, 20, 30, 40, 50, 60]

    println("Orijinal dizi:", sayilar);
    println("sayilar[1..4]:", d1);
    println("sayilar[3..]:", d2);
    println("sayilar[..3]:", d3);
    println("sayilar[..]:", d4);

    // 2. Dilim İndeksleme ve Uzunluk
    println("d1[0]:", d1[0]); // 20
    println("d1[1]:", d1[1]); // 30
    println("d1[2]:", d1[2]); // 40
    println("len(d1):", len(d1));
    println("d1.len():", d1.len());

    // 3. Alt Dilimleme (Sub-slicing)
    let alt_dilim = d1[1..3]; // [30, 40]
    println("d1[1..3] alt dilim:", alt_dilim);

    // 4. Fonksiyona Dilim (&[int]) Aktarımı
    let t1 = dizi_toplami(sayilar);       // Tüm diziyi dilim olarak aktarma
    let t2 = dizi_toplami(sayilar[1..4]); // Özel aralığı dilim olarak aktarma
    println("Tum dizi toplami:", t1);     // 210
    println("d1 (20+30+40) toplami:", t2); // 90

    // 5. Dilim Üzerinde For Döngüsü
    print("d1 elemanlari dongu: ");
    for x in d1 {
        print(x, " ");
    }
    println("");

    // 6. Dilimi Bağımsız Yeni Diziye Çevirme (to_array)
    let mut kopya = d1.to_array();
    push(kopya, 999);
    println("d1.to_array() + push(999):", kopya);
    println("Orijinal d1 degismedi:", d1);

    // 7. Metin Dilimleri (&str / String Slices)
    let metin = "merhaba oxide";
    let kelime1 = metin[0..7]; // "merhaba"
    let kelime2 = metin[8..];  // "oxide"
    let baslangic = metin[..7];
    let tum_metin = metin[..];

    println("Metin:", metin);
    println("metin[0..7]:", kelime1);
    println("metin[8..]:", kelime2);
    println("metin[..7]:", baslangic);
    println("metin[..]:", tum_metin);
    println("kelime1 uzunlugu (len):", len(kelime1));
    println("kelime2.to_upper():", kelime2.to_upper());

    // 8. String İnterpolasyonunda Dilim Kullanımı
    let formatli = f"Dilimlenen: '{kelime1}', Uzunluk: {len(kelime1)}, Dil: '{kelime2}'";
    println(formatli);

    println("=== Tum Dilim Testleri Basariyla Tamamlandi! ===");
}
