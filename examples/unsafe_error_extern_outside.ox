// examples/unsafe_error_extern_outside.ox
extern "C" {
    fn sqrt(x: float) -> float;
}

fn main() {
    let s = sqrt(9.0); // HATA: unsafe blok dışında çağrılamaz!
    println("s:", s);
}
