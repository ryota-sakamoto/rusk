// EXPECTED: 5050
fn f(a: i32) -> i32 {
    let b = 0;
    while a > 0 {
        b = b + a;
        a = a - 1;
    }

    return b;
}

fn main() {
    printf("%d", f(100));
}
