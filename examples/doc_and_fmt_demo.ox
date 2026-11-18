// Oxide Geliştirici Araçları Demo: oxide doc & oxide fmt (Aşama 25)

/// Sistem sürümü ve çekirdek adı
const SURUM: str = "v0.20.0";

/// İki boyutlu uzayda bir noktayı temsil eder.
struct Nokta {
    x: int,
    y: int,
}

/// Çizim ve ekranda görselleştirme yeteneği sunan temel arayüz.
trait Cizilebilir {
    /// Nesneyi ekrana çizer veya formatlar.
    fn ciz(self) -> str;
}

impl Cizilebilir for Nokta {
    fn ciz(self) -> str {
        f"Nokta({self.x}, {self.y})"
    }
}

/// Sistem görev durumunu belirten enum.
enum GorevDurumu {
    Beklemede,
    Calisiyor,
    Tamamlandi,
}

/// İki sayıyı toplayarak sonucu döndürür.
///
/// # Argümanlar
/// * `a` - İlk tamsayı
/// * `b` - İkinci tamsayı
///
/// # Dönüş
/// Toplam değeri
fn topla(a: int, b: int) -> int {
    a + b
}

fn main() {
    println("=== Oxide Geliştirici Araçları (Aşama 25) Demo ===");
    let p = Nokta { x: 10, y: 20 };
    println("Çizilen nokta:", p.ciz());

    let toplam = topla(15, 27);
    println("Toplam sonucu:", toplam);
    assert(toplam == 42);

    println("Sistem Sürümü:", SURUM);
    println("=== Demo Başarıyla Tamamlandı ===");
}
