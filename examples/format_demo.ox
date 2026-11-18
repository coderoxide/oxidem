// Oxide String Formatlama & Metin İnterpolasyonu Demo (Aşama 15)

fn main() {
    println("=== Oxide String Formatlama & Metin İnterpolasyonu (Aşama 15) ===");

    // 1. Temel f"..." İnterpolasyonu
    let ad = "Can";
    let dil = "Oxide";
    let versiyon = 1;
    let s1 = f"Gelistirici: {ad}, Dil: {dil} v{versiyon}";
    println(s1);

    // 2. İfade (Expression) İçeren İnterpolasyon
    let x = 15;
    let y = 25;
    let s2 = f"{x} + {y} = {x + y}, {x} * {y} = {x * y}";
    println(s2);

    // 3. String Metot Çağrıları ile İnterpolasyon
    let mesaj = "sistem programlama";
    let s3 = f"Orijinal: '{mesaj}', Buyuk: '{mesaj.to_upper()}'";
    println(s3);

    // 4. Farklı Tiplerin İnterpolasyonu (bool, float, char, Option, Result, Tuple)
    let aktif = true;
    let pi = 3.14;
    let bas_harf = 'O';
    let opt = Some(42);
    let demet = (100, "basarili", true);

    let s4 = f"Aktif: {aktif}, Pi: {pi}, Harf: {bas_harf}, Option: {opt}, Demet: {demet}";
    println(s4);

    // 5. Kaçışlı Süslü Parantezler (Escaped Braces {{ ve }})
    let sayi = 99;
    let s5 = f"JSON formati: {{\"deger\": {sayi}}}";
    println(s5);

    // 6. Rust-Tarzı format("...", args...) Fonksiyonu
    let kullanici = "Demir";
    let puan = 1500;
    let s6 = format("Oyuncu: {}, Puan: {}", kullanici, puan);
    println(s6);

    // 7. Formatlı Metinleri Birleştirme (+ operatörü)
    let s7 = f"Sonuc: {x * 2}" + " [TAMAMLANDI]";
    println(s7);

    println("=== Tum String Formatlama Testleri Basariyla Tamamlandi! ===");
}
