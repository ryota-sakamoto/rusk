// EXPECTED: 100
fn main() {
    let mut a = 0;
    while a < 100 {
        a += 1;
    }

    printf("%d", a);
}
