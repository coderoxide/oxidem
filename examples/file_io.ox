fn main() {
    println("=== Oxide Dosya Giriş / Çıkış (I/O) Testi ===");

    let dosya_adi = "examples/test_io.txt";
    let icerik = "Merhaba Oxide!\nBu metin Oxide standart I/O kütüphanesi tarafından yazıldı ve okundu.";

    println("1. Dosyaya yazılıyor:", dosya_adi);
    let yazildi = write_file(dosya_adi, icerik);
    println("Yazma başarılı mı?:", yazildi);

    println("\n2. Dosyadan okunuyor...");
    let okunan = read_file(dosya_adi);
    println("Okunan içerik:\n---");
    println(okunan);
    println("---");
    println("Okunan metin uzunluğu (len):", len(okunan));

    println("\n=== Dosya I/O Testi Başarıyla Tamamlandı ===");
}
