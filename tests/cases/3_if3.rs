// EXPECTED: 55
fn f(n: i32) -> i32 {
    if n == 1 {
        return 1;
    } else if n == 2 {
        return 1;
    }

    return f(n - 1) + f(n - 2);
}

fn main() {
    printf("%d", f(10));
}
