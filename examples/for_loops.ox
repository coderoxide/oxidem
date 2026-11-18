fn main() {
    println("=== 1. For Range Döngüsü (0..5) ===");
    for i in 0..5 {
        println("i:", i);
    }

    println("\n=== 2. For Döngüsünde Break Testi (0..10) ===");
    for i in 0..10 {
        if i == 4 {
            println("4'e ulaşıldı, döngü sonlandırılıyor (break)!");
            break;
        }
        println("Adım:", i);
    }

    println("\n=== 3. For Döngüsünde Continue Testi (Tek Sayılar 0..6) ===");
    for i in 0..6 {
        if i % 2 == 0 {
            continue;
        }
        println("Tek sayı:", i);
    }

    println("\n=== 4. Koleksiyon Üzerinde For In Döngüsü ===");
    let sayilar = [100, 200, 300, 400];
    for num in sayilar {
        println("Eleman:", num);
    }

    println("\n=== 5. While Döngüsünde Break ve Continue ===");
    let mut sayac = 0;
    while sayac < 10 {
        sayac = sayac + 1;
        if sayac % 2 == 0 {
            continue;
        }
        if sayac > 7 {
            println("sayac > 7 oldu, döngüden çıkılıyor!");
            break;
        }
        println("While döngüsü tek sayı:", sayac);
    }

    println("\n=== Testler Başarıyla Tamamlandı ===");
}
