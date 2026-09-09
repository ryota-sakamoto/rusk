// EXPECTED: 789123
fn main() {
    let mut a = 123;
    {
        let mut a = 456;
        a = 789;
        printf("%d", a);
    }

    printf("%d", a);
}
