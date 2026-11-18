# Oxide Veri Yapıları Rehberi (Data Structures)

Oxide, zengin veri modelleme imkanları sunar: Dinamik Diziler, Rust benzeri `struct` yapıları ve metotları organize eden `impl` blokları.

---

## 1. Dinamik Diziler (Arrays)

Oxide dizileri `[eleman1, eleman2, ...]` sözdizimi ile tanımlanır.

### Dizi Oluşturma ve Okuma
```rust
let sayilar = [10, 20, 30, 40];
println("İlk eleman:", sayilar[0]); // 10
println("İkinci eleman:", sayilar[1]); // 20
println("Uzunluk:", len(sayilar)); // 4
```

### Dizi Elemanlarını Değiştirme ve Mutability Kuralı
Dizi elemanlarının güncellenebilmesi için dizinin `let mut` ile tanımlanmış olması zorunludur:

```rust
let mut liste = [1, 2, 3];
liste[0] = 99; // Geçerli! liste: [99, 2, 3]

let sabit_liste = [1, 2, 3];
// sabit_liste[0] = 5; // HATA! Değiştirilemez 'sabit_liste' dizisine eleman atanamaz!
```

### Diziye Eleman Ekleme ve Çıkarma
```rust
let mut sehirler = ["Ankara", "İstanbul"];
push(sehirler, "İzmir");
println("Yeni Şehirler:", sehirler); // ["Ankara", "İstanbul", "İzmir"]

let son_sehir = pop(sehirler);
println("Çıkarılan:", son_sehir); // "İzmir"
```

---

## 2. Yapılar (`struct`)

`struct`, birbiriyle ilişkili verileri adlandırılmış alanlar (fields) altında gruplar.

### Struct Tanımlama
```rust
struct Point {
    x: int,
    y: int,
}

struct User {
    name: str,
    age: int,
    is_active: bool,
}
```

### Struct Başlatma (Instantiation)
```rust
let mut p = Point {
    x: 10,
    y: 20,
};

let user = User {
    name: "Can",
    age: 25,
    is_active: true,
};
```

### Alan Erişimi ve Değişmezlik Koruması
```rust
println("X koordinatı:", p.x);

// Eğer 'let mut' ile tanımlandıysa alan değiştirilebilir:
p.x = 45;
println("Yeni X:", p.x); // 45

// user sabit olduğu için:
// user.age = 26; // HATA! Değiştirilemez 'user' nesnesinin alanları değiştirilemez!
```

---

## 3. Uygulama Blokları (`impl`) ve Metotlar

Oxide'da veri yapıları (`struct`) ile davranışlar (`metotlar`) Rust'taki gibi birbirinden ayrılmıştır. Bir struct'a metot kazandırmak için `impl` bloğu kullanılır.

### `impl` Tanımlama ve `self`
Metodun ilk parametresi `self` (veya `mut self`) olduğunda, o metot struct örneği üzerinden çağrılabilir:

```rust
struct Rectangle {
    width: int,
    height: int,
}

impl Rectangle {
    // Alan hesabı yapan metot
    fn area(self) -> int {
        self.width * self.height
    }

    // Genişliği artıran metot
    fn scale_width(self, factor: int) {
        self.width = self.width * factor;
    }
}
```

### Metot Çağırma
```rust
fn main() {
    let mut rect = Rectangle {
        width: 10,
        height: 5,
    };

    println("Alan:", rect.area()); // 50

    rect.scale_width(2);
    println("Yeni Genişlik:", rect.width); // 20
    println("Yeni Alan:", rect.area()); // 100
}
```
