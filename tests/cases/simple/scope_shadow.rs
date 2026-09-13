// EXPECTED: 789123
fn main() {
    let a = 123;
    {
        let a = false;
        if a {
            printf("%d", 456);
        } else {
            printf("%d", 789);
        }
    }

    printf("%d", a);
}
