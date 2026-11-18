// Oxide Hata Yönetimi Demo (Aşama 12)
// Option<T>, Result<T, E>, Some, None, Ok, Err ve '?' (Try) Operatörü

fn guvenli_bol_opt(pay: int, payda: int) -> Option<int> {
    if payda == 0 {
        return None;
    }
    return Some(pay / payda);
}

fn hesapla_opt(a: int, b: int, carpim: int) -> Option<int> {
    // '?' operatörü: eğer None ise fonksiyondan None döner, Some ise açar (unwrap)
    let bolum = guvenli_bol_opt(a, b)?;
    return Some(bolum * carpim);
}

fn pozitif_bol_res(a: int, b: int) -> Result<int, str> {
    if b == 0 {
        return Err("Sifira bolme hatasi!");
    }
    if a < 0 || b < 0 {
        return Err("Negatif sayi kabul edilmez!");
    }
    return Ok(a / b);
}

fn zincir_islem_res(x: int, y: int) -> Result<int, str> {
    // '?' operatörü: Err ise hemen Err fırlatır/döner, Ok ise içeriği çıkarır
    let b1 = pozitif_bol_res(x, y)?;
    let b2 = pozitif_bol_res(b1, 2)?;
    return Ok(b2 + 10);
}

fn main() {
    println("=== Oxide Hata Yönetimi Demo (Aşama 12) ===");

    // 1. Option<T> Temel Kullanımı
    println("--- 1. Option Metotları ---");
    let o1: Option<int> = Some(42);
    let o2: Option<int> = None;

    println("o1.is_some():", o1.is_some());
    println("o1.is_none():", o1.is_none());
    println("o1.unwrap():", o1.unwrap());
    println("o2.is_some():", o2.is_some());
    println("o2.is_none():", o2.is_none());
    println("o2.unwrap_or(999):", o2.unwrap_or(999));

    // 2. Result<T, E> Temel Kullanımı
    println("--- 2. Result Metotları ---");
    let r1: Result<int, str> = Ok(100);
    let r2: Result<int, str> = Err("Baglanti hatasi");

    println("r1.is_ok():", r1.is_ok());
    println("r1.is_err():", r1.is_err());
    println("r1.unwrap():", r1.unwrap());
    println("r2.is_ok():", r2.is_ok());
    println("r2.is_err():", r2.is_err());
    println("r2.unwrap_or(0):", r2.unwrap_or(0));

    // 3. Match ile Desen Eşleme
    println("--- 3. Pattern Matching (Some / None / Ok / Err) ---");
    let test_opt = Some(88);
    match test_opt {
        Some(deger) => {
            println("Option degeri yakalandi:", deger);
        }
        None => {
            println("Option bos!");
        }
    }

    let test_res: Result<int, str> = Err("Yetki reddedildi");
    match test_res {
        Ok(sayi) => {
            println("Basarili sonuc:", sayi);
        }
        Err(hata) => {
            println("Hata mesaji yakalandi:", hata);
        }
    }

    // 4. '?' (Try) Operatörü ile Akış Kontrolü
    println("--- 4. '?' (Try) Operatoru ---");
    let opt_ok = hesapla_opt(100, 5, 2);
    println("hesapla_opt(100, 5, 2).unwrap():", opt_ok.unwrap());

    let opt_fail = hesapla_opt(100, 0, 2);
    println("hesapla_opt(100, 0, 2).is_none():", opt_fail.is_none());

    let res_ok = zincir_islem_res(100, 5);
    println("zincir_islem_res(100, 5).unwrap():", res_ok.unwrap());

    let res_err1 = zincir_islem_res(100, 0);
    println("zincir_islem_res(100, 0).is_err():", res_err1.is_err());

    let res_err2 = zincir_islem_res(-50, 5);
    println("zincir_islem_res(-50, 5).is_err():", res_err2.is_err());

    println("=== Tum Hata Yonetimi Testleri Basariyla Tamamlandi! ===");
}
