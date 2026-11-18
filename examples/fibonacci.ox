fn fib(n: int) -> int {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fn main() {
    println("--- Fibonacci Serisi Hesabı ---");
    let mut i = 0;
    while i <= 10 {
        println("fib(" + i + ") =", fib(i));
        i = i + 1;
    }
}
