// Oxide Derleme Zamanı Sabitleri (const) ve Dosya/Dizin Sistemi (std::fs, std::env) Demo (Aşama 20)

// 1. Global Derleme Zamanı Sabitleri (Compile-Time Evaluated Constants)
const BUFFER_SIZE: int = 1024 * 4;
const PI: float = 3.14159;
const APP_NAME: str = "OxideFS";
const GREETING: str = APP_NAME + " Sistem Araci";

fn test_constants() {
    println("--- 1. Derleme Zamanı Sabitleri (const) ---");
    println("BUFFER_SIZE (1024 * 4):", BUFFER_SIZE);
    println("PI:", PI);
    println("GREETING:", GREETING);

    // Blok içi yerel const
    const CARPAN: int = 10 + 2;
    let sonuc = BUFFER_SIZE / CARPAN;
    println("BUFFER_SIZE / CARPAN (4096 / 12):", sonuc);
}

fn test_filesystem() {
    println("--- 2. Standart Dosya Sistemi (fs::) ---");
    let dosya_yolu = "test_io_temp.txt";

    // Dosyaya yazma
    let icerik = "Merhaba! Oxide guvenli sistem programlama dili dosya sistemi testi.";
    let yazma_sonuc = fs::write(dosya_yolu, icerik);
    if yazma_sonuc.is_ok() {
        println("Dosya yazma: Basarili");
    } else {
        println("Dosya yazma hatasi olustu");
    }

    // Dosya varlık kontrolü
    let var_mi = fs::exists(dosya_yolu);
    println("Dosya mevcut mu?:", var_mi);

    // Dosyadan okuma
    let okuma_sonuc = fs::read_to_string(dosya_yolu);
    match okuma_sonuc {
        Ok(metin) => {
            println("Okunan icerik:", metin);
        }
        Err(hata) => {
            println("Okuma hatasi:", hata);
        }
    }

    // Dosya silme
    let silme_sonuc = fs::remove_file(dosya_yolu);
    if silme_sonuc.is_ok() {
        println("Dosya silme: Basarili");
    }

    let son_durum = fs::exists(dosya_yolu);
    println("Silme sonrasi mevcut mu?:", son_durum);
}

fn test_environment() {
    println("--- 3. Sistem & Çevre Değişkenleri (env::) ---");

    // Komut satırı argümanları
    let cli_args = env::args();
    if len(cli_args) >= 1 {
        println("Argumanlar basariyla alindi: true");
    }

    // Çevre değişkeni kontrolü
    let path_var = env::var("PATH");
    if path_var.is_some() {
        println("PATH cevre degiskeni: Mevcut (is_some: true)");
    } else {
        println("PATH cevre degiskeni bulunamadi");
    }

    // Çalışma dizini
    let pwd_res = env::current_dir();
    if pwd_res.is_ok() {
        println("Calisma dizini basariyla alindi (is_ok: true)");
    }
}

fn main() {
    println("=== Oxide Aşama 20: Const & Dosya/Dizin Sistemi Demo ===");
    test_constants();
    test_filesystem();
    test_environment();
    println("=== Tum Aşama 20 Testleri Basariyla Tamamlandi! ===");
}
