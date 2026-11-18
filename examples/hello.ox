fn greet(name: str) {
    println("Selam, " + name + "! Oxide dünyasına hoş geldin.");
}

fn add(a: int, b: int) -> int {
    a + b
}

fn main() {
    greet("Dostum");

    let x = 15;
    let y = 27;
    let total = add(x, y);

    println("Toplam:", total);
}
