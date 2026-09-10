// EXPECTED: 40
enum Test {
    A,
    B(i32, i32, i32),
}

fn main() {
    let t = Test::B(10, 15, 20);
    let mut result = 0;
    match t {
        Test::A => {
            result = 1;
        }
        Test::B(a, b, c) => {
            result = c * 2;
        }
    }

    printf("%d", result);
}
