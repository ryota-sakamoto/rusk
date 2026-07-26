// EXPECTED: 23
fn f() -> i32 {
    let a = 5;
    let b = 4;
    return a * (b + 3);
}
fn main() {
    printf("%d", f() - 12);
    return 0;
}
