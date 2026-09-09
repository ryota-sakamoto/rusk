// EXPECTED: 20
enum Test {
    A,
    B(i32),
}

fn main() {
    let a = Test::B(10);
    let mut b = 0;
    match a {
        Test::A => {
            b = 1;
        }
        Test::B(n) => {
            b = n * 2;
        }
    }

    printf("%d", b);
}
