// EXPECTED: 789
enum Test {
    A,
    B,
    C,
}

fn main() {
    let t = Test::C;
    match t {
        Test::A => {
            printf("%d", 123);
        }
        Test::B => {
            printf("%d", 456);
        }
        _ => {
            printf("%d", 789);
        }
    }
}
