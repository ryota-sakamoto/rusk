// EXPECTED: 5050
fn f(n: i32) -> i32 {
    if !(n == 0) {
        return n + f(n - 1);
    }
    return n;
}

fn main() {
    printf("%d", f(100));
}
