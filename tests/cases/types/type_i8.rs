// EXPECTED: 9
fn f(a: i8, b: i8) -> i8 {
    return a * (b - 2);
}

fn main() {
    printf("%d", f(3, 5));
}
