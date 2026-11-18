struct Vector2 {
    x: int,
    y: int,
}

impl Vector2 {
    fn new(x: int, y: int) -> Vector2 {
        Vector2 { x: x, y: y }
    }

    fn topla(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

fn topla(a: int, b: int) -> int {
    a + b
}

fn carp(a: int, b: int) -> int {
    a * b
}

fn faktoriyel(n: int) -> int {
    if n <= 1 {
        1
    } else {
        n * faktoriyel(n - 1)
    }
}
