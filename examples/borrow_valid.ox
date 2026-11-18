// examples/borrow_valid.ox
// Oxide dilinde Referanslar, Non-Lexical Lifetimes (NLL) ve Borrow Checker testi

fn main() {
    println("=== 1. Paylaşılmış Referanslar (&T) Testi ===");
    let mut x: int = 100;
    
    // Birden fazla paylaşılan referans aynı anda bulunabilir:
    let r1: &int = &x;
    let r2: &int = &x;
    println("r1 degeri:", *r1);
    println("r2 degeri:", *r2);

    // r1 ve r2'nin son kullanımı yukarıda bitti (Non-Lexical Lifetimes - NLL)!
    // Artık x üzerinde tekil ve değiştirilebilir (&mut) referans alınabilir:
    println("=== 2. Değiştirilebilir Referans (&mut T) ve NLL Testi ===");
    let r_mut: &mut int = &mut x;
    println("r_mut uzerinden okuma:", *r_mut);

    // r_mut'un kullanımı bitti, x tekrar serbest:
    x = 250;
    println("x'in guncel degeri:", x);

    println("=== 3. Unsafe Blok ve FFI Güvenlik Testi ===");
    unsafe {
        println("Unsafe blok calisiyor!");
    }

    println("=== Borrow Checker & NLL Basariyla Tamamlandi ===");
}
