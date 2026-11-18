import "math_lib.ox";

fn main() {
    println("=== Çoklu Dosya & Modül Sistemi Testi ===");

    // İçe aktarılan fonksiyonlar
    let toplam = topla(15, 25);
    let carpim = carp(6, 7);
    let f5 = faktoriyel(5);

    println("topla(15, 25) =", toplam);
    println("carp(6, 7)    =", carpim);
    println("faktoriyel(5) =", f5);

    // İçe aktarılan struct ve metodlar
    let v1 = Vector2 { x: 10, y: 20 };
    let v2 = Vector2 { x: 30, y: 40 };
    let v3 = v1.topla(v2);

    println("v1.x:", v1.x, "v1.y:", v1.y);
    println("v2.x:", v2.x, "v2.y:", v2.y);
    println("v1 + v2 sonucu -> x:", v3.x, "y:", v3.y);

    println("=== Modül Testi Başarıyla Tamamlandı ===");
}
