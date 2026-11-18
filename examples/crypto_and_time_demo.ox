fn test_crypto_and_hashing() -> void {
    println("--- 1. Kriptografik Özet & Doğrulama (Hash & CRC32) ---");
    let metin = "merhaba oxide guvenlik ve sistem programlama";
    let ozet = crypto::hash(metin);
    println("Metin Özeti (FNV-1a 64-bit):", ozet);
    assert(len(ozet) == 16, "Hash 16 hex karakter olmali");

    let ayni_ozet = crypto::hash(metin);
    assert(ozet == ayni_ozet, "Deterministik hash eslesmeli");

    let farkli_ozet = crypto::hash("farkli veri");
    assert(ozet != farkli_ozet, "Farkli veri farkli hash uretmeli");

    let crc = crypto::crc32("123456789");
    println("CRC32 Kontrol Toplamı:", crc);
    assert(crc != 0, "CRC32 degeri hesaplanmali");
}

fn test_base64_encoding() -> void {
    println("--- 2. Base64 Kodlama & Çözme ---");
    let orijinal = "Oxide Dili Sistem Programlama 2026!";
    let kodlanmis = base64::encode(orijinal);
    println("Base64 Kodlanmış:", kodlanmis);

    let cozulen = base64::decode(kodlanmis).unwrap();
    println("Base64 Çözülmüş:", cozulen);
    assert(cozulen == orijinal, "Cozulen metin orijinal ile birebir eslesmeli");

    let bos = base64::encode("");
    assert(bos == "", "Bos metin encode bos donmeli");
    assert(base64::decode("").unwrap() == "", "Bos decode bos donmeli");

    let hatali = base64::decode("%%%gecersiz!");
    assert(hatali.is_err(), "Gecersiz base64 hata vermeli");
}

fn test_timing_and_instant() -> void {
    println("--- 3. Zamanlama & Kronometre (time & Instant) ---");
    let epoch_ms = time::now_millis();
    let epoch_s = time::now_secs();
    println("Unix Zaman Damgası -> ms:", epoch_ms, ", s:", epoch_s);
    assert(epoch_ms > 0 && epoch_s > 0, "Unix zamani pozitif olmali");

    let kronometre = Instant::now();
    thread::sleep(60);

    let gecen_ms = kronometre.elapsed_millis();
    let gecen_us = kronometre.elapsed_micros();
    println("Kronometre -> Geçen süre:", gecen_ms, "ms (", gecen_us, "mikrosaniye)");
    assert(gecen_ms >= 45, "En az 45ms uyumus olmali");
    assert(gecen_us >= 45000, "En az 45000 mikrosaniye gecmis olmali");
}

fn main() {
    println("=== Oxide Kriptografi, Kodlama & Zamanlama (Aşama 24) ===");
    test_crypto_and_hashing();
    test_base64_encoding();
    test_timing_and_instant();
    println("=== Tum Kripto, Base64 ve Zamanlama Testleri Basariyla Tamamlandi! ===");
}
