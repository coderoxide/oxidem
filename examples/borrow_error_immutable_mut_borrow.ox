// examples/borrow_error_immutable_mut_borrow.ox
fn main() {
    let x = 10;
    let r = &mut x; // HATA: Değişmez 'x' değişkeni &mut ile ödünç alınamaz!
    println("r:", *r);
}
