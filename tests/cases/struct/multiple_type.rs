// EXPECTED: 32
struct Test {
    a: i32,
    b: bool,
}

fn f(t: Test) -> i32 {
    if t.b {
        return t.a * 8;
    }
    return t.a;
}

fn main() {
    let a = Test { a: 4, b: true };
    printf("%d", f(a));
}
