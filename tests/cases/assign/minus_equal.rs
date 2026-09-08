// EXPECTED: 50
fn main() {
    let mut a = 100;
    while a > 50 {
        a -= 1;
    }

    printf("%d", a);
}
