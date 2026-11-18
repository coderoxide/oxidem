// examples/chars_and_bytes.ox
// Oxide dilinde 'char' ve 'u8' ilkel tipleri testi

fn test_chars() {
    let c1: char = 'O';
    let c2 = 'x';
    println("c1:", c1, "c2:", c2);

    let ascii_kod = to_int(c1);
    println("c1 ('O') ASCII kodu:", ascii_kod);

    let harf = to_char(ascii_kod + 1);
    println("ascii_kod + 1 harfi:", harf);

    if c1 < c2 {
        println("'O' < 'x' karsilastirmasi basarili!");
    }

    match c1 {
        'A' => println("A bulundu"),
        'O' => println("Desen eslesmesi: 'O' basariyla yakalandi!"),
        _ => println("Diger karakter"),
    }
}

fn test_bytes() {
    let b1: u8 = b'A';
    let b2: u8 = 10;
    let b3 = b1 + b2;
    println("b1 (b'A'):", b1, "b2:", b2, "b3 (b1 + b2):", b3);

    let as_char = to_char(b1);
    println("b1 karakter olarak:", as_char);

    match b1 {
        b'B' => println("B bayti"),
        b'A' => println("Desen eslesmesi: b'A' basariyla yakalandi!"),
        _ => println("Diger bayt"),
    }
}

fn main() {
    println("=== Oxide Char ve U8 Testi ===");
    test_chars();
    test_bytes();
    println("=== Char & U8 Testi Basariyla Tamamlandi ===");
}
