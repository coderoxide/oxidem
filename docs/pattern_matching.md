# Oxide Desen Eşleştirme Rehberi (Pattern Matching)

Desen eşleştirme (Pattern Matching), Oxide'ın en güçlü denetim akışı mekanizmasıdır. `enum` tipleri ve `match` ifadesi ile tip güvenli, okunabilir ve hatasız kod yazılmasını sağlar.

---

## 1. Numaralandırmalar (`enum`)

`enum`, bir değişkenin alabileceği sonlu durumlar kümesini tanımlar.

```rust
enum Direction {
    North,
    South,
    East,
    West,
}

enum ResultStatus {
    Success,
    Warning,
    Error,
}
```

### Varyantlara Erişim (`::` Çift İki Nokta Sözdizimi)
Rust'ta olduğu gibi bir enum varyantına `EnumIsmi::Varyant` ile erişilir:

```rust
let yon = Direction::North;
let durum = ResultStatus::Success;
```

---

## 2. `match` İfadesi

`match`, verilen bir hedef değeri (subject) tanımlanan desenlerle (arms) yukarıdan aşağıya doğru sırayla karşılaştırır. Eşleşen ilk dalın gövdesi yürütülür.

### Sözdizimi
```rust
match hedef {
    desen1 => ifade1,
    desen2 => ifade2,
    _ => varsayilan_ifade,
}
```

---

## 3. Desen Türleri

### 3.1. Enum Varyantı Eşleştirme
```rust
fn yon_mesaji(d: Direction) -> str {
    match d {
        Direction::North => "Kuzeye doğru gidiyorsun.",
        Direction::South => "Güneye yöneldin.",
        Direction::East  => "Doğuya bakıyorsun.",
        Direction::West  => "Batıdasın.",
    }
}
```

### 3.2. Sabit Değer (Literal) Eşleştirme
Sayılar, metinler ve boolean değerler doğrudan eşleştirilebilir:

```rust
let kod = 404;

let hata = match kod {
    200 => "Tamam",
    400 => "Hatalı İstek",
    404 => "Sayfa Bulunamadı",
    500 => "Sunucu Hatası",
    _   => "Bilinmeyen Hata",
};
```

### 3.3. Wildcard (`_`) Deseni
`_` deseni kapsanmayan tüm diğer durumları yakalar (varsayılan dal):

```rust
match x {
    0 => println("Sıfır"),
    _ => println("Sıfırdan farklı bir sayı"),
}
```

### 3.4. Değişken Bağlama (Variable Binding)
Eşleşen değer, bir desen değişkenine atanarak dal içerisinde kullanılabilir:

```rust
let mut sayi = 10;

match sayi {
    1 => println("Bir"),
    2 => println("İki"),
    diger => println("Beklenmeyen değer yakalandı: " + diger),
}
```

---

## 4. İfade Olarak `match` (Değer Üretimi)

Oxide'da `match` bir ifadedir. Bu nedenle doğrudan bir değişkene atanabilir veya bir fonksiyonun dönüş değeri olabilir:

```rust
fn durum_puani(s: ResultStatus) -> int {
    match s {
        ResultStatus::Success => 100,
        ResultStatus::Warning => 50,
        ResultStatus::Error   => 0,
    }
}
```

---

## 5. Eşleşme Güvenliği (Exhaustiveness Check)

Eğer `match` ifadesindeki dallardan hiçbiri verilen değerle eşleşmezse, derleyici programın belirsiz duruma düşmesini engellemek için satır ve sütun bilgisiyle çalışma zamanı hatası üretir:

```text
Hata [3:5]: 'match' ifadesinde eşleşen hiçbir desen (arm) bulunamadı! Değer: 99
```
Bu nedenle tüm varyantların kapsanmadığı durumlarda wildcard (`_`) veya değişken bağlama (`diger => ...`) kullanılması önerilir.
