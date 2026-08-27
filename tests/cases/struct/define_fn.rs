// EXPECTED: 7
struct Test {}

impl Test {
    fn f() -> i32 {
        return 7;
    }
}

fn main() {
    printf("%d", Test::f());
}
