// EXPECTED: 2
enum Test {
    A,
    B,
    C,
}

fn main() {
    let a = Test::C;
    printf("%d", a);
}
