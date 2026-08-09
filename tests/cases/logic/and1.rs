// EXPECTED: 200
fn f(a: i32) -> i32 {
    if a >= 10 && a <= 30 {
        return 5;
    }
    return a;
}

fn main() {
    printf("%d", f(40) * f(20));
}
