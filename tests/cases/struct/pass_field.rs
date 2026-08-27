// EXPECTED: 15
struct Test {
    b: bool,
}

fn f(b: bool) -> i32 {
    if b {
        return 3;
    }
    return 5;
}

fn main() {
    let a = Test { b: true };
    let c = Test { b: false };
    printf("%d", f(a) * f(c));
}
