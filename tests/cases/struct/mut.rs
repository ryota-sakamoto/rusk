// EXPECTED: 11
struct Test {
    a: i32,
    b: i32,
    c: i32,
}

fn main() {
    let mut test = Test { a: 1, c: 3, b: 2 };
    test.a = 5;
    printf("%d", test.a + test.b * test.c);
}
