extern "C" {
    fn sqrt(x: float) -> float;
    fn pow(base: float, exp: float) -> float;
    fn sin(x: float) -> float;
    fn cos(x: float) -> float;
    fn abs(x: int) -> int;
}

fn main() {
    println("=== Oxide C FFI (Foreign Function Interface) Testi ===");

    unsafe {
        let karekok = sqrt(49.0);
        println("sqrt(49.0) =", karekok);

        let kuvvet = pow(2.0, 8.0);
        println("pow(2.0, 8.0) =", kuvvet);

        let sinus = sin(0.0);
        let kosinus = cos(0.0);
        println("sin(0.0) =", sinus);
        println("cos(0.0) =", kosinus);

        let mutlak = abs(-99);
        println("abs(-99) =", mutlak);
    }

    println("=== C FFI Başarıyla Tamamlandı ===");
}
