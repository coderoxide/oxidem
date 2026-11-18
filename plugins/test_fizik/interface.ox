// Oxide Arayüz Başlığı: test_fizik
// Kullanım: import "plugins/test_fizik/interface.ox";

extern "C" {
    fn test_fizik_selam() -> int;
    fn test_fizik_topla(a: int, b: int) -> int;
    fn test_fizik_mesafe_kare(x1: float, y1: float, x2: float, y2: float) -> float;
}
