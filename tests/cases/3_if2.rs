// EXPECTED: 5
fn f(n: i32) -> i32 {
    if (n == 1) {
        return 3;
    } else {
        return 5;
    }
    return 0;
}

fn main() {
    printf("%d", f(2));
    return 0;
}
