// EXPECTED: 55
fn f(a: i32) -> i32 {
    if a == 1 || a == 2 {
        return 1;
    }
    return f(a - 1) + f(a - 2);
}

fn main() {
    printf("%d", f(10));
}
