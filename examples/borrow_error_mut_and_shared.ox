// examples/borrow_error_mut_and_shared.ox
fn main() {
    let mut x = 10;
    let r1 = &x;
    let r2 = &mut x; // HATA: 'x' paylaşılmışken &mut alınamaz!
    println("r1:", *r1);
    println("r2:", *r2);
}
