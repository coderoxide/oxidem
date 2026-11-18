fn test_string_methods() -> void {
    println("=== 1. String Metotları Testi ===");
    let raw = "   merhaba oxide dunya   ";
    let temiz = raw.trim();
    println("Trim:", temiz);

    let buyuk = temiz.to_upper();
    println("Buyuk harf:", buyuk);

    let kucuk = buyuk.to_lower();
    println("Kucuk harf:", kucuk);

    let var_mi = temiz.contains("oxide");
    println("Contains 'oxide':", var_mi);

    let degismis = temiz.replace("dunya", "evren");
    println("Replace:", degismis);

    let ilk_harf = temiz.char_at(0);
    println("Ilk harf (char_at 0):", ilk_harf);

    let parcalar = degismis.split(" ");
    println("Split uzunlugu:", parcalar.len());
    println("Parca 0:", parcalar[0]);
    println("Parca 1:", parcalar[1]);
    println("Parca 2:", parcalar[2]);
}

fn test_array_methods() -> void {
    println("=== 2. Dizi Metotları Testi ===");
    let mut sayilar = [10, 20, 30];
    println("Baslangic uzunlugu:", sayilar.len());

    sayilar.push(40);
    sayilar.push(50);
    println("Push sonrasi uzunluk:", sayilar.len());

    let son = sayilar.pop();
    println("Pop edilen eleman:", son);
    println("Son uzunluk:", sayilar.len());
}

fn test_map() -> void {
    println("=== 3. Map (Hash Tablosu) Testi ===");
    let m = map_new();
    m.set("ad", "Oxide");
    m.set("tur", "Sistem Dili");
    m.set("surum", "1.0");

    println("Map uzunlugu:", m.len());
    println("m.get('ad'):", m.get("ad"));
    println("m.get('tur'):", m.get("tur"));
    println("m.has('ad'):", m.has("ad"));
    println("m.has('olmayan'):", m.has("olmayan"));

    let anahtarlar = m.keys();
    println("Anahtar sayisi:", anahtarlar.len());
}

fn test_system_api() -> void {
    println("=== 4. Sistem ve Cevre Degiskenleri (System API) ===");
    let cli_args = args();
    println("Arguman sayisi:", cli_args.len());

    let path_val = get_env("PATH");
    let path_dolu_mu = path_val.len() > 0;
    println("PATH cevre degiskeni var mi?:", path_dolu_mu);
}

fn main() -> void {
    println("=== Oxide Standard Library (Aşama 8) Demo ===");
    test_string_methods();
    test_array_methods();
    test_map();
    test_system_api();
    println("=== Tum Aşama 8 Testleri Basariyla Tamamlandi! ===");
}
