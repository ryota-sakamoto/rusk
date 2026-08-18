// EXPECTED: 7
struct Test {
    a: i32,
    b: i32,
    c: i32,
}

fn main() {
    let test: Test = Test { a: 1, c: 3, b: 2 };
    printf("%d", test.a + test.b * test.c);
}
