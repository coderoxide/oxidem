enum TrafficLight {
    Red,
    Yellow,
    Green,
}

enum Status {
    Pending,
    InProgress,
    Completed,
    Failed,
}

fn describe_light(light: TrafficLight) -> str {
    match light {
        TrafficLight::Red => "Dur! Kırmızı ışık yanıyor.",
        TrafficLight::Yellow => "Hazırlan! Sarı ışık yanıyor.",
        TrafficLight::Green => "Geç! Yeşil ışık yanıyor.",
        _ => "Geçersiz ışık!",
    }
}

fn handle_status(s: Status) {
    match s {
        Status::Pending => println("[DURUM] Görev sırada bekliyor..."),
        Status::InProgress => println("[DURUM] Görev şu an işleniyor..."),
        Status::Completed => println("[DURUM] Tebrikler! Görev başarıyla bitti."),
        Status::Failed => println("[DURUM] Hata! Görev başarısız oldu."),
    }
}

fn main() {
    println("=== Oxide Enum & Match (Pattern Matching) Testi ===");

    // 1. Enum ve Match İfadesi (Değer Döndüren)
    let isik = TrafficLight::Green;
    let mesaj = describe_light(isik);
    println("Trafik Işığı Durumu:", mesaj);

    let kirmizi = TrafficLight::Red;
    println("Trafik Işığı Durumu:", describe_light(kirmizi));

    // 2. Status Enum Testi
    println("\n--- Görev Durumları ---");
    handle_status(Status::InProgress);
    handle_status(Status::Completed);

    // 3. Sayı ve Wildcard ile Match İfadesi
    println("\n--- Sayı Eşleştirme (Pattern Matching) ---");
    let mut sayi = 1;
    while sayi <= 4 {
        let yazi = match sayi {
            1 => "Birinci",
            2 => "İkinci",
            3 => "Üçüncü",
            diger => "Diğer sıra (" + diger + ")",
        };
        println("Sıra:", yazi);
        sayi = sayi + 1;
    }
}
