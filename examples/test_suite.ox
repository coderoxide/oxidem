fn test_toplama() -> void {
    let a = 15;
    let b = 25;
    assert(a + b == 40, "Toplam 40 olmali");
}

fn test_string_metotlari() -> void {
    let s = "  oxide  ";
    let t = s.trim();
    assert(t == "oxide", "Trim hatali");
    assert(t.len() == 5, "Uzunluk 5 olmali");
    assert(t.contains("id"), "Contains 'id' icermeli");
}

fn test_map_islemleri() -> void {
    let m = map_new();
    m.set("dil", "Oxide");
    m.set("hiz", "Yuksek");
    assert(m.len() == 2, "Map eleman sayisi 2 olmali");
    assert(m.has("dil"), "dil anahtari bulunmali");
    assert(m.get("dil") == "Oxide", "dil degeri Oxide olmali");
}

fn test_dizi_islemleri() -> void {
    let mut dizi = [1, 2, 3];
    dizi.push(4);
    assert(dizi.len() == 4, "Dizi boyutu 4 olmali");
    let son = dizi.pop();
    assert(son == 4, "Son eleman 4 olmali");
    assert(dizi.len() == 3, "Pop sonrasi boyut 3 olmali");
}
