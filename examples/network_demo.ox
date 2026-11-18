fn main() {
    println("=== Oxide Ağ & Soket Programlama (Aşama 22) ===");

    let adres = "127.0.0.1:4545";

    // 1. Sunucu Thread'i Başlatılıyor
    let sunucu_handle = thread::spawn(|| {
        println("Sunucu: Port dinleniyor (127.0.0.1:4545)...");
        let listener = TcpListener::bind("127.0.0.1:4545").unwrap();

        println("Sunucu: Istemci baglantisi bekleniyor...");
        let client_stream = listener.accept().unwrap();

        println("Sunucu: Istemci baglandi! Veri okunuyor...");
        let gelen_mesaj = client_stream.read().unwrap();
        println("Sunucu Alinan Veri:", gelen_mesaj);

        println("Sunucu: Yanit gonderiliyor...");
        client_stream.write("PONG: Merhaba Istemci!");
        client_stream.close();
        0
    });

    // Sunucunun ayağa kalkması için kısa bir bekleme
    thread::sleep(120);

    // 2. İstemci Bağlantısı
    println("Istemci: Sunucuya baglaniliyor...");
    let stream = TcpStream::connect("127.0.0.1:4545").unwrap();

    println("Istemci: Veri gonderiliyor (PING)...");
    let gonderilen_bayt = stream.write("PING: Merhaba Sunucu!").unwrap();
    println("Istemci Gonderilen Bayt Sayisi:", gonderilen_bayt);

    println("Istemci: Sunucudan yanit bekleniyor...");
    let yanit = stream.read().unwrap();
    println("Istemci Alinan Yanit:", yanit);
    assert(yanit == "PONG: Merhaba Istemci!", "Sunucu yaniti eslesmeli");

    stream.close();
    sunucu_handle.join().unwrap();

    println("=== Tum Ag & Soket Testleri Basariyla Tamamlandi! ===");
}
