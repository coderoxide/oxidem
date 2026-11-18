// Oxide Yerel Rust Eklentisi: test_fizik
// Bu kütüphane Oxide içerisinden sıfır maliyetli C-ABI ile çağrılır.

#[unsafe(no_mangle)]
pub extern "C" fn test_fizik_selam() -> i64 {
    println!("[Oxide Rust Plugin: test_fizik] Eklenti başarıyla aktif edildi!");
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn test_fizik_topla(a: i64, b: i64) -> i64 {
    a + b
}

#[unsafe(no_mangle)]
pub extern "C" fn test_fizik_mesafe_kare(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    dx * dx + dy * dy
}

