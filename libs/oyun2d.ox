/// Oxide 2D Oyun Motoru, Vektör Matematiği ve Çizim Kütüphanesi (oyun2d)
/// Bu kütüphane herhangi bir Oxide projesinden `import "oyun2d";` denilerek kullanılabilir.

extern "C" {
    fn sqrt(x: float) -> float;
}

/// 2 Boyutlu Uzayda Vektör Temsili
struct Vec2 {
    x: float,
    y: float,
}

fn vec2(x: float, y: float) -> Vec2 {
    Vec2 { x: x, y: y }
}

impl Vec2 {
    fn topla(self, diger: Vec2) -> Vec2 {
        Vec2 { x: self.x + diger.x, y: self.y + diger.y }
    }

    fn cikar(self, diger: Vec2) -> Vec2 {
        Vec2 { x: self.x - diger.x, y: self.y - diger.y }
    }

    fn carp_skalar(self, s: float) -> Vec2 {
        Vec2 { x: self.x * s, y: self.y * s }
    }

    fn uzunluk_kare(self) -> float {
        self.x * self.x + self.y * self.y
    }

    fn uzunluk(self) -> float {
        let sq = self.uzunluk_kare();
        unsafe {
            sqrt(sq)
        }
    }

    fn nokta_carpim(self, diger: Vec2) -> float {
        self.x * diger.x + self.y * diger.y
    }

    fn info(self) -> str {
        f"Vec2({self.x}, {self.y})"
    }
}

/// 2 Boyutlu Eksen Hizalı Sınır Kutusu (AABB - Axis Aligned Bounding Box)
struct Rect {
    x: float,
    y: float,
    w: float,
    h: float,
}

fn rect(x: float, y: float, w: float, h: float) -> Rect {
    Rect { x: x, y: y, w: w, h: h }
}

impl Rect {
    fn alan(self) -> float {
        self.w * self.h
    }

    /// Verilen koordinat kutunun içinde mi?
    fn nokta_icerir(self, px: float, py: float) -> bool {
        px >= self.x && px <= (self.x + self.w) && py >= self.y && py <= (self.y + self.h)
    }

    /// Başka bir kutuyla çarpışıyor mu? (AABB Çarpışma Testi)
    fn carpisir_mi(self, diger: Rect) -> bool {
        let carpisma_x = (self.x + self.w) >= diger.x && (diger.x + diger.w) >= self.x;
        let carpisma_y = (self.y + self.h) >= diger.y && (diger.y + diger.h) >= self.y;
        carpisma_x && carpisma_y
    }

    fn info(self) -> str {
        f"Rect(x: {self.x}, y: {self.y}, w: {self.w}, h: {self.h})"
    }
}

/// Renk Temsili (RGB)
struct Renk {
    r: int,
    g: int,
    b: int,
}

impl Renk {
    fn yeni(r: int, g: int, b: int) -> Renk {
        Renk { r: r, g: g, b: b }
    }

    /// Terminal için ANSI 24-bit TrueColor formatı üretir
    fn ansi_onplan(self) -> str {
        f"\x1b[38;2;{self.r};{self.g};{self.b}m"
    }

    fn ansi_sifirla() -> str {
        "\x1b[0m"
    }
}

/// 2D Karakter Tabanlı Yazılımsal Framebuffer & Tuval Motoru
struct Tuval {
    genislik: int,
    yukseklik: int,
    pikseller: [str],
}

fn tuval_yeni(g: int, y: int, zemin: str) -> Tuval {
    let mut arr: [str] = [];
    let toplam = g * y;
    let mut i = 0;
    while i < toplam {
        arr.push(zemin);
        i = i + 1;
    }
    Tuval { genislik: g, yukseklik: y, pikseller: arr }
}

impl Tuval {
    fn temizle(mut self, zemin: str) {
        let mut i = 0;
        let toplam = self.genislik * self.yukseklik;
        while i < toplam {
            self.pikseller[i] = zemin;
            i = i + 1;
        }
    }

    fn piksel_koy(mut self, x: int, y: int, c: str) {
        if x >= 0 && x < self.genislik && y >= 0 && y < self.yukseklik {
            let idx = y * self.genislik + x;
            self.pikseller[idx] = c;
        }
    }

    fn kutu_ciz(mut self, rx: int, ry: int, rw: int, rh: int, c: str) {
        let mut j = ry;
        while j < ry + rh {
            let mut i = rx;
            while i < rx + rw {
                self.piksel_koy(i, j, c);
                i = i + 1;
            }
            j = j + 1;
        }
    }

    fn cerceve_ciz(mut self, rx: int, ry: int, rw: int, rh: int) {
        let mut i = rx;
        while i < rx + rw {
            self.piksel_koy(i, ry, "═");
            self.piksel_koy(i, ry + rh - 1, "═");
            i = i + 1;
        }
        let mut j = ry;
        while j < ry + rh {
            self.piksel_koy(rx, j, "║");
            self.piksel_koy(rx + rw - 1, j, "║");
            j = j + 1;
        }
        self.piksel_koy(rx, ry, "╔");
        self.piksel_koy(rx + rw - 1, ry, "╗");
        self.piksel_koy(rx, ry + rh - 1, "╚");
        self.piksel_koy(rx + rw - 1, ry + rh - 1, "╝");
    }

    fn render(self) -> str {
        let mut out = "";
        let mut y = 0;
        while y < self.yukseklik {
            let mut x = 0;
            while x < self.genislik {
                let idx = y * self.genislik + x;
                out = out + self.pikseller[idx];
                x = x + 1;
            }
            out = out + "\n";
            y = y + 1;
        }
        out
    }
}
