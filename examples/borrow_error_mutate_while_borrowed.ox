// examples/borrow_error_mutate_while_borrowed.ox
fn main() {
    let mut x = 10;
    let r = &x;
    x = 20; // HATA: 'x' referansla ödünç alınmışken doğrudan değiştirilemez!
    println("r:", *r);
}
