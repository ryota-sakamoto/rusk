// EXPECTED: 120
enum Test {
    A,
    B,
    C,
}

fn main() {
    let a = Test::C;
    let mut b = 0;
    match a {
        Test::A => {
            b = 1;
        }
        Test::B => {
            b = 2;
        }
        Test::C => {
            b = 3;
        }
    }

    printf("%d", b * 40);
}
