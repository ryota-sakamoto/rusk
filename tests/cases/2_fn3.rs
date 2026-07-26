// EXPECTED: 64
fn f(a: i32, b: i32, c: i32) {
    let d = (a - b) * c;
    return d * 4;
}
fn main() {
    printf("%d", f(7, 3, 4));
    return 0;
}
