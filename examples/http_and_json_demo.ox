fn test_json_operations() -> void {
    println("--- 1. JSON Serileştirme & Çözümleme ---");
    let m = map_new();
    m.set("isim", "Oxide");
    m.set("tur", "Sistem Programlama Dili");
    m.set("surum", "v0.19.0");

    let json_metni = json::stringify(m);
    println("json::stringify sonucu:", json_metni);

    let cozulen = json::parse(json_metni).unwrap();
    println("json::parse cozulen isim:", cozulen.get("isim"));
    println("json::parse cozulen surum:", cozulen.get("surum"));
    assert(cozulen.get("isim") == "Oxide", "JSON isim eslesmeli");
    assert(cozulen.get("surum") == "v0.19.0", "JSON surum eslesmeli");

    let hizli_tur = json::get(json_metni, "tur").unwrap();
    println("json::get 'tur':", hizli_tur);
    assert(hizli_tur == "Sistem Programlama Dili", "Hizli key extraction eslesmeli");

    let olmayan = json::get(json_metni, "bilinmeyen");
    assert(olmayan.is_none(), "Olmayan anahtar None donmeli");
}

fn test_http_server_and_client() -> void {
    println("--- 2. HTTP Sunucusu & Istemcisi ---");
    let adres = "127.0.0.1:4546";

    // Sunucuyu arka planda baslat
    let sunucu_handle = thread::spawn(|| {
        let listener = TcpListener::bind("127.0.0.1:4546").unwrap();

        // 1. Istek: GET
        let client1 = listener.accept().unwrap();
        let raw_req1 = client1.read().unwrap();
        let method1 = http::request_method(raw_req1);
        let path1 = http::request_path(raw_req1);
        println("Sunucu Aldı (1): Metot =", method1, ", Yol =", path1);
        assert(method1 == "GET", "Metot GET olmali");
        assert(path1 == "/api/status", "Yol /api/status olmali");

        let body1 = "{\"status\": \"OK\", \"msg\": \"Oxide HTTP Sunucusu Aktif\"}";
        let resp1 = http::response(200, "application/json", body1);
        client1.write(resp1).unwrap();
        client1.close();

        // 2. Istek: POST
        let client2 = listener.accept().unwrap();
        let raw_req2 = client2.read().unwrap();
        let method2 = http::request_method(raw_req2);
        let path2 = http::request_path(raw_req2);
        let req_body2 = http::request_body(raw_req2);
        println("Sunucu Aldı (2): Metot =", method2, ", Yol =", path2, ", Govde =", req_body2);
        assert(method2 == "POST", "Metot POST olmali");
        assert(path2 == "/api/echo", "Yol /api/echo olmali");

        let gonderilen_kullanici = json::get(req_body2, "kullanici").unwrap();
        let body2 = "{\"sonuc\": \"Eklendi\", \"user\": \"" + gonderilen_kullanici + "\"}";
        let resp2 = http::response(201, "application/json", body2);
        client2.write(resp2).unwrap();
        client2.close();

        0
    });

    thread::sleep(120);

    // 1. GET Cagrisi
    println("Istemci: GET /api/status gonderiliyor...");
    let get_yanit = http::get("http://127.0.0.1:4546/api/status").unwrap();
    println("Istemci Alinan GET Govdesi:", get_yanit);
    let durum = json::get(get_yanit, "status").unwrap();
    println("Istemci Yanit Durumu:", durum);
    assert(durum == "OK", "GET yanit status OK olmali");

    // 2. POST Cagrisi
    println("Istemci: POST /api/echo gonderiliyor...");
    let post_veri = "{\"kullanici\": \"Can\", \"rol\": \"admin\"}";
    let post_yanit = http::post("http://127.0.0.1:4546/api/echo", post_veri, "application/json").unwrap();
    println("Istemci Alinan POST Govdesi:", post_yanit);
    let sonuc = json::get(post_yanit, "sonuc").unwrap();
    let echo_user = json::get(post_yanit, "user").unwrap();
    println("Istemci POST Onay:", sonuc, "User:", echo_user);
    assert(sonuc == "Eklendi", "POST sonuc Eklendi olmali");
    assert(echo_user == "Can", "POST echo user Can olmali");

    sunucu_handle.join().unwrap();
}

fn main() {
    println("=== Oxide HTTP Sunucusu, REST API & JSON Desteği (Aşama 23) ===");
    test_json_operations();
    test_http_server_and_client();
    println("=== Tum HTTP ve JSON Testleri Basariyla Tamamlandi! ===");
}
