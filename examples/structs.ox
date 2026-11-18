struct Player {
    name: str,
    score: int,
    level: int,
}

impl Player {
    fn add_score(self, points: int) {
        self.score = self.score + points;
        println(self.name + " puan kazandı! Yeni puan:", self.score);
    }

    fn info(self) {
        println("--- Oyuncu Bilgisi ---");
        println("İsim:", self.name);
        println("Seviye:", self.level);
        println("Puan:", self.score);
    }
}

fn main() {
    println("=== Oxide Struct & Impl Testi ===");

    // Struct oluşturma
    let mut p1 = Player {
        name: "Ahmet",
        score: 100,
        level: 2,
    };

    // Metot çağırma (self ile)
    p1.info();

    // Alan erişimi ve değiştirme
    println("\nDoğrudan alan okuma -> p1.score:", p1.score);
    p1.score = 250;
    println("p1.score = 250 ataması sonrası:", p1.score);

    // Metot ile puan artırma
    println("\nMetot çağrısı yapılıyor:");
    p1.add_score(75);

    p1.info();
}
