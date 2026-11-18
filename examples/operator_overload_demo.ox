// Oxide Operatör Aşırı Yükleme Demo (Aşama 17)
// +, -, *, /, - (unary), ==, !=, [] (index), [] = (index_set)

struct Vector2D {
    x: int,
    y: int,
}

trait Add {
    fn add(self, other: Vector2D) -> Vector2D;
}

trait Sub {
    fn sub(self, other: Vector2D) -> Vector2D;
}

trait Mul {
    fn mul(self, factor: int) -> Vector2D;
}

trait Div {
    fn div(self, factor: int) -> Vector2D;
}

trait Neg {
    fn neg(self) -> Vector2D;
}

trait Eq {
    fn eq(self, other: Vector2D) -> bool;
}

trait Index {
    fn index(self, idx: int) -> int;
}

impl Add for Vector2D {
    fn add(self, other: Vector2D) -> Vector2D {
        Vector2D {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vector2D {
    fn sub(self, other: Vector2D) -> Vector2D {
        Vector2D {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul for Vector2D {
    fn mul(self, factor: int) -> Vector2D {
        Vector2D {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

impl Div for Vector2D {
    fn div(self, factor: int) -> Vector2D {
        Vector2D {
            x: self.x / factor,
            y: self.y / factor,
        }
    }
}

impl Neg for Vector2D {
    fn neg(self) -> Vector2D {
        Vector2D {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Eq for Vector2D {
    fn eq(self, other: Vector2D) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Index for Vector2D {
    fn index(self, idx: int) -> int {
        if idx == 0 {
            self.x
        } else {
            self.y
        }
    }

    fn index_set(mut self, idx: int, val: int) {
        if idx == 0 {
            self.x = val;
        } else {
            self.y = val;
        }
    }
}

fn topla_generic<T: Add>(a: T, b: T) -> T {
    a.add(b)
}

fn main() {
    println("=== Oxide Operatör Aşırı Yükleme Demo (Aşama 17) ===");

    let v1 = Vector2D { x: 10, y: 20 };
    let v2 = Vector2D { x: 3, y: 7 };

    // 1. Binary + (Add)
    let v_toplam = v1 + v2;
    println("v1 + v2 -> x:", v_toplam.x, "y:", v_toplam.y);

    // 2. Binary - (Sub)
    let v_fark = v1 - v2;
    println("v1 - v2 -> x:", v_fark.x, "y:", v_fark.y);

    // 3. Binary * (Mul)
    let v_carp = v1 * 3;
    println("v1 * 3 -> x:", v_carp.x, "y:", v_carp.y);

    // 4. Binary / (Div)
    let v_bol = v1 / 2;
    println("v1 / 2 -> x:", v_bol.x, "y:", v_bol.y);

    // 5. Unary - (Neg)
    let v_neg = -v1;
    println("-v1 -> x:", v_neg.x, "y:", v_neg.y);

    // 6. Zincirleme İfadeler (Chaining)
    let v_zincir = (v1 + v2) * 2;
    println("(v1 + v2) * 2 -> x:", v_zincir.x, "y:", v_zincir.y);

    // 7. Karşılaştırma: == ve != (Eq)
    let v3 = Vector2D { x: 10, y: 20 };
    println("v1 == v3:", v1 == v3);
    println("v1 == v2:", v1 == v2);
    println("v1 != v2:", v1 != v2);
    println("v1 != v3:", v1 != v3);

    // 8. İndeksleme: [] (Index) ve [] = (IndexSet)
    println("v1[0]:", v1[0]);
    println("v1[1]:", v1[1]);

    let mut v_mut = Vector2D { x: 50, y: 60 };
    v_mut[0] = 555;
    v_mut[1] = 666;
    println("v_mut index_set sonrasi -> [0]:", v_mut[0], "[1]:", v_mut[1]);

    // 9. Generic Fonksiyon ile Trait Bound (<T: Add>)
    let v_gen = topla_generic(v1, v2);
    println("topla_generic(v1, v2) -> x:", v_gen.x, "y:", v_gen.y);

    println("=== Tum Operator Asiri Yukleme Testleri Basariyla Tamamlandi! ===");
}
