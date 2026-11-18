// Oxide Akıllı İşaretçiler ve Özyinelemeli Veri Yapıları Demosu (Aşama 19)

struct Node {
    val: int,
    next: Option<Box<Node>>,
}

impl Node {
    fn topla(self) -> int {
        let mut toplam = self.val;
        let mut curr = self.next;
        while curr.is_some() {
            let n = curr.unwrap();
            toplam = toplam + n.val;
            curr = n.next;
        }
        toplam
    }

    fn goster(self) -> void {
        print(self.val);
        let mut curr = self.next;
        while curr.is_some() {
            let n = curr.unwrap();
            print(" -> ");
            print(n.val);
            curr = n.next;
        }
        println(" -> Son");
    }
}

fn test_box_temel() -> void {
    println("--- 1. Box<T> Temel Yığın Tahsisi ve Dereferans ---");
    let b = Box::new(42);
    println("Box degeri (*b):", *b);
    assert(*b == 42, "b 42 olmali");

    let mut b_mut = Box::new(100);
    *b_mut = 250;
    println("b_mut guncel degeri (*b_mut):", *b_mut);
    assert(*b_mut == 250, "b_mut 250 olmali");
}

fn test_recursive_linked_list() -> void {
    println("--- 2. Özyinelemeli Bağlı Liste (Recursive LinkedList: Node) ---");
    // 10 -> 20 -> 30 -> Son
    let n3 = Node { val: 30, next: None };
    let n2 = Node { val: 20, next: Some(Box::new(n3)) };
    let list = Node { val: 10, next: Some(Box::new(n2)) };

    print("Bagli liste: ");
    list.goster();

    let liste_toplam = list.topla();
    println("Liste elemanlari toplami:", liste_toplam);
    assert(liste_toplam == 60, "Toplam 60 olmali (10 + 20 + 30)");

    // Doğrudan Box auto-deref alan erişimi
    let mut ikinci_box = list.next.unwrap();
    println("Ikinci dugum degeri (auto-deref n.val):", ikinci_box.val);
    assert(ikinci_box.val == 20, "Ikinci dugum 20 olmali");

    // Box üzerinden alan ataması (auto-deref field assign)
    ikinci_box.val = 88;
    println("Ikinci dugum guncellendi:", ikinci_box.val);
    assert(ikinci_box.val == 88, "Ikinci dugum 88 olmali");
}

fn test_rc_shared_pointers() -> void {
    println("--- 3. Rc<T> Paylaşımlı Referans Sayımı ---");
    let rc1 = Rc::new(777);
    println("rc1 degeri (*rc1):", *rc1);
    let sayac1 = Rc::strong_count(rc1);
    println("rc1 baslangic referans sayisi:", sayac1);
    assert(sayac1 == 1, "Ilk sayac 1 olmali");

    let rc2 = rc1.clone();
    let sayac2 = Rc::strong_count(rc1);
    println("rc2 klonlandi, guncel sayac:", sayac2);
    assert(sayac2 == 2, "Klonlama sonrasi sayac 2 olmali");

    println("rc2 uzerinden okuma (*rc2):", *rc2);
    assert(*rc1 == 777, "rc1 777 olmali");
    assert(*rc2 == 777, "rc2 777 olmali");
}

fn test_refcell_interior_mutability() -> void {
    println("--- 4. RefCell<T> İçsel Değiştirilebilirlik ---");
    let cell = RefCell::new(50);
    println("cell ilk hali (*cell.borrow()):", *cell.borrow());
    assert(*cell.borrow() == 50, "cell ilk degeri 50 olmali");

    let mut b = cell.borrow_mut();
    *b = 999;
    println("cell.borrow_mut() ile guncellendi: 999");
    assert(*cell.borrow() == 999, "cell guncel degeri 999 olmali");
}

fn main() -> void {
    println("=== Oxide Akıllı İşaretçiler (Smart Pointers - Aşama 19) ===");
    test_box_temel();
    test_recursive_linked_list();
    test_rc_shared_pointers();
    test_refcell_interior_mutability();
    println("=== Tum Akilli Isaretci Testleri Basariyla Tamamlandi! ===");
}
