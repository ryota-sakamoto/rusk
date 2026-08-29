// EXPECTED: 11
struct Test {
    a: i32,
    b: i32,
}

impl Test {
    fn sum(self) -> i32 {
        return self.a + self.b;
    }
}

fn main() {
    let t = Test { a: 4, b: 7 };
    printf("%d", t.sum());
}
