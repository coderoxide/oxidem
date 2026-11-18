// Oxide 2D Oyun Motoru ve Kütüphane Demosu
// 'oyun2d' kütüphanesi doğrudan 'libs/' dizininden import edilir:
import "oyun2d";

fn main() {
    println("==================================================");
    println("🎮 OXIDE 2D OYUN MOTORU & KÜTÜPHANE SİSTEMİ DEMO");
    println("==================================================");

    // 1. Vektör Matematiği Testi
    let v1 = vec2(10.0, 20.0);
    let v2 = vec2(5.0, 10.0);
    let v3 = v1.topla(v2);
    let v4 = v1.cikar(v2);
    let v_skalar = v1.carp_skalar(2.0);
    let nokta = v1.nokta_carpim(v2);
    let mesafe_345 = vec2(3.0, 4.0).uzunluk();

    println("Vektör 1:", v1.info());
    println("Vektör 2:", v2.info());
    println("Toplam Vektör:", v3.info());
    println("Fark Vektör:", v4.info());
    println("2x Skalar Çarpım:", v_skalar.info());
    println("Nokta Çarpımı (Dot Product):", nokta);
    println("Hipotenüs Uzunluğu (3-4-5 Üçgeni):", mesafe_345);
    assert(mesafe_345 == 5.0);

    // 2. Çarpışma Testi (AABB Collision)
    let oyuncu = rect(10.0, 10.0, 20.0, 20.0);
    let dusman = rect(25.0, 15.0, 15.0, 15.0);
    let engel = rect(100.0, 100.0, 30.0, 30.0);

    println("\n--- Çarpışma Testi (AABB) ---");
    println("Oyuncu Kutusu:", oyuncu.info());
    println("Düşman Kutusu:", dusman.info());
    let carpisti_mi = oyuncu.carpisir_mi(dusman);
    let engele_degdi_mi = oyuncu.carpisir_mi(engel);

    println("Oyuncu ve Düşman Çarpışıyor mu?:", carpisti_mi);
    println("Oyuncu ve Uzak Engel Çarpışıyor mu?:", engele_degdi_mi);
    assert(carpisti_mi == true);
    assert(engele_degdi_mi == false);

    // 3. 2D Framebuffer & Sahne Çizimi (Tuval)
    println("\n--- 2D Terminal Sahne Render ---");
    let mut sahne = tuval_yeni(30, 10, " ");
    // Çerçeve çiz
    sahne.cerceve_ciz(0, 0, 30, 10);
    // Oyuncu karakteri çiz
    sahne.kutu_ciz(4, 3, 3, 2, "█");
    // Hedef/Bayrak çiz
    sahne.piksel_koy(24, 5, "★");
    sahne.piksel_koy(25, 5, "⚑");

    let gorunum = sahne.render();
    print(gorunum);

    println("==================================================");
    println("✔ Oyun Motoru Kütüphanesi Başarıyla Çalıştırıldı!");
    println("==================================================");
}
