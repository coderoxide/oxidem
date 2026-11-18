struct Point {
    x: int,
    y: int,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    println("Nokta:", p.x, p.y);
    p.x = 99; // HATA: p immutable tanımlandı!
}
