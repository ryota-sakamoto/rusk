// EXPECTED: 30
enum Test {
    A(i32),
    B(bool),
}

fn main() {
    let a = Test::A(10);
    let b = Test::B(false);

    let mut result = 0;
    match a {
        Test::A(n) => {
            result = n;
        }
        _ => {}
    }

    match b {
        Test::B(bb) => {
            if bb {
                result *= 2;
            } else {
                result *= 3;
            }
        }
        _ => {}
    }

    printf("%d", result);
}
