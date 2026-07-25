// EXPECTED: 23
fn f() {
    let a = 5;
    let b = 4;
    return a * (b + 3);
}
fn main() {
    printf(f() - 12);
    return 0;
}
