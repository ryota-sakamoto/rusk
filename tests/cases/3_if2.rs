// EXPECTED: 5
fn f(n) {
    if (n == 1) {
        return 3;
    } else {
        return 5;
    }
    return 0;
}

fn main() {
    printf(f(2));
    return 0;
}
