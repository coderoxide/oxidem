fn test_math() -> int {
    let a = 10 + 20 * 2;
    let b = a - 15;
    let sonuc = if true {
        b + 5
    } else {
        999
    };
    sonuc
}

fn main() -> void {
    let x = test_math();
    println("Optimizasyon sonucu:", x);
}
