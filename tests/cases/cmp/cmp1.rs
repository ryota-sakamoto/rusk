// EXPECTED: 14
fn f(a: i32) -> i32 {
    if a > 100 {
        return 1;
    } else if a >= 50 {
        return 2;
    } else if a <= 10 {
        return 3;
    } else if a < 2 {
        return 4;
    }
    return a;
}
fn main() {
    printf("%d", f(101) + f(100) + f(50) + f(10) + f(2) + f(1));
}

