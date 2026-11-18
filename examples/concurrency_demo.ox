fn main() {
    println("=== Oxide Çoklu İş Parçacığı & Eşzamanlılık (Aşama 21) ===");

    // 1. Thread Spawn & Join Testi
    println("--- 1. Thread Spawn ve Join ---");
    let handle = thread::spawn(|| {
        println("Thread 1: Arka planda calisiyor...");
        thread::sleep(50);
        println("Thread 1: Is tamamlandi, sonuc donuyor.");
        42
    });

    let res = handle.join().unwrap();
    println("Ana Thread: Thread 1'den donen sonuc:", res);
    assert(res == 42, "Thread donus degeri 42 olmali");

    // 2. Channel (Kanal) Mesajlaşma Testi (Producer - Consumer)
    println("--- 2. Channel (Mesajlasma) ---");
    let (tx, rx) = channel();

    let sender_thread = thread::spawn(|| {
        println("Uretici Thread: Mesaj gonderiliyor: 100");
        tx.send(100);
        thread::sleep(30);
        println("Uretici Thread: Mesaj gonderiliyor: 200");
        tx.send(200);
        0
    });

    let msg1 = rx.recv().unwrap();
    println("Tuketici Thread (Ana): Alinan mesaj 1:", msg1);
    assert(msg1 == 100, "Ilk mesaj 100 olmali");

    let msg2 = rx.recv().unwrap();
    println("Tuketici Thread (Ana): Alinan mesaj 2:", msg2);
    assert(msg2 == 200, "Ikinci mesaj 200 olmali");

    sender_thread.join().unwrap();

    // 3. Mutex Paylaşımlı Veri ve Kilitleme Testi
    println("--- 3. Mutex (Paylasimli Erisim & Kilitleme) ---");
    let sayac = Mutex::new(10);
    {
        let lock_guard = sayac.lock();
        *lock_guard = *lock_guard + 15;
    }
    let son_deger = *sayac.lock();
    println("Mutex kilit sonrasi sayac degeri:", son_deger);
    assert(son_deger == 25, "Mutex degeri 25 olmali");

    println("=== Tum Concurrency & Multi-threading Testleri Basariyla Tamamlandi! ===");
}
