// Oxide Trait ve Arayüz Mimarisi Demo (Aşama 13)
// trait, impl Trait for Struct, varsayılan metot gövdeleri ve <T: Trait> sınırları

trait Goster {
    fn goster(self) -> str;
    
    // Varsayılan gövdeye sahip metot (Default Method):
    fn ozet(self) -> str {
        return "Gosterilen: " + self.goster();
    }
}

trait Alan {
    fn alan(self) -> int;
}

struct Nokta {
    x: int,
    y: int,
}

impl Goster for Nokta {
    fn goster(self) -> str {
        return "Nokta(" + self.x + ", " + self.y + ")";
    }
    // 'ozet' metodunu varsayılan olarak Goster trait'inden devralır!
}

struct Dikdortgen {
    en: int,
    boy: int,
}

impl Alan for Dikdortgen {
    fn alan(self) -> int {
        return self.en * self.boy;
    }
}

impl Goster for Dikdortgen {
    fn goster(self) -> str {
        return "Dikdortgen(" + self.en + "x" + self.boy + ")";
    }

    // Varsayılan 'ozet' metodunu ezer (Override):
    fn ozet(self) -> str {
        return "Ozel Ozet -> " + self.goster();
    }
}

// Trait Bound ile Generic Fonksiyon: T sadece 'Goster' trait'ini uygulayan tipler olabilir
fn bilgi_yazdir<T: Goster>(nesne: T) {
    println("Bilgi:", nesne.goster());
    println("Ozet:", nesne.ozet());
}

fn main() {
    println("=== Oxide Trait ve Arayüz Mimarisi (Aşama 13) ===");

    // 1. Nokta struct'ı ve Trait metotları
    println("--- 1. Nokta (Varsayılan Trait Metodu Mirası) ---");
    let p = Nokta { x: 10, y: 20 };
    println("p.goster():", p.goster());
    println("p.ozet():", p.ozet());

    // 2. Dikdortgen struct'ı ve Çoklu Trait Uygulaması
    println("--- 2. Dikdortgen (Alan & Goster Traitleri) ---");
    let d = Dikdortgen { en: 5, boy: 8 };
    println("d.alan():", d.alan());
    println("d.goster():", d.goster());
    println("d.ozet():", d.ozet());

    // 3. Trait Bound ile Generic Fonksiyon Çağrısı (Statik Gönderim / Monomorphization)
    println("--- 3. Trait Bound ile Generic Fonksiyon ---");
    bilgi_yazdir(p);
    bilgi_yazdir(d);

    println("=== Tum Trait Testleri Basariyla Tamamlandi! ===");
}
