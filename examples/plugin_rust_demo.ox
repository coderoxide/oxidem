import "plugins/test_fizik/interface.ox";

fn main() {
    println("=== Oxide & Rust Native Plugin Demo ===");
    unsafe {
        let s = test_fizik_selam();
        println("Eklenti selam sonucu:", s);

        let toplam = test_fizik_topla(100, 250);
        println("Rust fonksiyonu toplam:", toplam);
        assert(toplam == 350);

        let dist_sq = test_fizik_mesafe_kare(0.0, 0.0, 3.0, 4.0);
        println("Fizik mesafe kare (3-4-5 üçgeni):", dist_sq);
    }

    println("=== Rust Eklenti Testi Başarıyla Tamamlandı! ===");
}
